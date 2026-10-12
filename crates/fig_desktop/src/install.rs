use std::path::Path;

use fig_integrations::Integration;
use fig_integrations::input_method::InputMethod;
use fig_integrations::shell::ShellExt;
use fig_os_shim::Context;
use fig_util::consts::{
    APP_BUNDLE_ID,
    APP_PROCESS_NAME,
    CLI_BINARY_NAME,
    LAUNCH_AGENT_LABEL,
    PTY_BINARY_NAME,
};
use fig_util::directories::fig_data_dir;
use fig_util::launchd_plist::{
    LaunchdPlist,
    create_launch_agent,
};
use fig_util::Shell;
use macos_utils::bundle::{
    get_bundle_path,
    get_bundle_path_for_executable,
};
use semver::Version;
use tracing::{
    error,
    warn,
};

use crate::utils::is_cargo_debug_build;

const PREVIOUS_VERSION_KEY: &str = "desktop.versionAtPreviousLaunch";

fn run_input_method_migration() {
    use tokio::time::{
        Duration,
        sleep,
    };

    let input_method = InputMethod::default();
    match input_method.target_bundle_path() {
        Ok(target_bundle_path) if target_bundle_path.exists() => {
            tokio::spawn(async move {
                input_method.terminate().ok();
                if let Err(err) = input_method.migrate().await {
                    warn!(%err, "Failed to migrate input method");
                }

                sleep(Duration::from_secs(1)).await;
                input_method.launch();
            });
        },
        Ok(_) => warn!("Input method bundle path does not exist"),
        Err(err) => warn!(%err, "Failed to get input method bundle path"),
    }
}

/// Run items at launch
pub async fn run_install() {
    if let Err(err) = initialize_fig_dir(&fig_os_shim::Env::new()).await {
        error!(%err, "Failed to initialize data dir");
    }

    // Add any items that are only once per version
    if should_run_install_script() {
        run_input_method_migration();
    }

    if let Err(err) = set_previous_version(current_version()) {
        error!(%err, "Failed to set previous version");
    }
}

/// Whether the dotfile, accessibility, and input method setup steps are all done
pub async fn setup_complete(ctx: &Context, accessibility_enabled: bool) -> bool {
    if !accessibility_enabled || InputMethod::default().is_installed().await.is_err() {
        return false;
    }

    for shell in Shell::supported() {
        match shell.get_shell_integrations(ctx.env()) {
            Ok(integrations) => {
                for integration in integrations {
                    if integration.is_installed().await.is_err() {
                        return false;
                    }
                }
            },
            Err(_) => return false,
        }
    }

    true
}

/// Symlink, and overwrite if it already exists and is invalid or not a symlink
async fn symlink(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), std::io::Error> {
    use std::io::ErrorKind;

    let src = src.as_ref();
    let dst = dst.as_ref();

    // Check if the link already exists
    match tokio::fs::symlink_metadata(dst).await {
        Ok(metadata) => {
            // If it's a symlink, check if it points to the right place
            if metadata.file_type().is_symlink() {
                if let Ok(read_link) = tokio::fs::read_link(dst).await {
                    if read_link == src {
                        return Ok(());
                    }
                }
            }

            // If it's not a symlink or it points to the wrong place, delete it
            tokio::fs::remove_file(dst).await?;
        },
        Err(err) if err.kind() == ErrorKind::NotFound => {},
        Err(err) => return Err(err),
    }

    // Create the symlink
    tokio::fs::symlink(src, dst).await
}

pub async fn initialize_fig_dir(env: &fig_os_shim::Env) -> anyhow::Result<()> {
    let local_bin = fig_util::directories::home_local_bin()?;
    if let Err(err) = std::fs::create_dir_all(&local_bin) {
        error!(%err, "Failed to create {local_bin:?}");
    }

    // Install figterm to ~/.local/bin
    match get_bundle_path_for_executable(PTY_BINARY_NAME) {
        Some(pty_path) => {
            let link = local_bin.join(PTY_BINARY_NAME);
            if let Err(err) = symlink(&pty_path, link).await {
                error!(%err, "Failed to symlink for {PTY_BINARY_NAME}: {pty_path:?}");
            }

            for shell in Shell::supported() {
                let pty_shell_cpy = local_bin.join(format!("{shell} ({PTY_BINARY_NAME})"));
                let pty_path = pty_path.clone();

                tokio::spawn(async move {
                    // Check version if copy already exists, this is because everytime a copy is made the first start is
                    // kinda slow and we want to avoid that
                    if pty_shell_cpy.exists() {
                        let output = tokio::process::Command::new(&pty_shell_cpy)
                            .arg("--version")
                            .output()
                            .await
                            .ok();

                        let version = output
                            .as_ref()
                            .and_then(|output| std::str::from_utf8(&output.stdout).ok())
                            .map(|s| {
                                match s.strip_prefix(PTY_BINARY_NAME) {
                                    Some(s) => s,
                                    None => s,
                                }
                                .trim()
                            });

                        if version == Some(env!("CARGO_PKG_VERSION")) {
                            return;
                        }
                    }

                    if let Err(err) = tokio::fs::remove_file(&pty_shell_cpy).await {
                        error!(%err, "Failed to remove {PTY_BINARY_NAME} shell {shell:?} copy");
                    }
                    if let Err(err) = tokio::fs::copy(&pty_path, &pty_shell_cpy).await {
                        error!(%err, "Failed to copy {PTY_BINARY_NAME} to {}", pty_shell_cpy.display());
                    }
                });
            }
        },
        None => error!("Failed to find {PTY_BINARY_NAME} in bundle"),
    }

    // install the cli to ~/.local/bin
    match get_bundle_path_for_executable(CLI_BINARY_NAME) {
        Some(cli_path) => {
            let dest = local_bin.join(CLI_BINARY_NAME);
            if let Err(err) = symlink(&cli_path, dest).await {
                error!(%err, "Failed to symlink {CLI_BINARY_NAME}");
            }
        },
        None => error!("Failed to find {CLI_BINARY_NAME} in bundle"),
    }

    if let Some(bundle_path) = get_bundle_path() {
        let exe = bundle_path.join("Contents").join("MacOS").join(APP_PROCESS_NAME);
        let startup_launch_agent = LaunchdPlist::new(LAUNCH_AGENT_LABEL)
            .program_arguments([&exe.to_string_lossy(), "--is-startup", "--no-dashboard"])
            .associated_bundle_identifiers([APP_BUNDLE_ID])
            .run_at_load(true);

        create_launch_agent(&startup_launch_agent)?;

        let path = startup_launch_agent.get_file_path()?;
        std::process::Command::new("launchctl")
            .arg("load")
            .arg(&path)
            .status()
            .ok();
    }

    // Init the shell directory
    std::fs::create_dir(fig_data_dir()?.join("shell")).ok();
    for shell in Shell::supported() {
        for script_integration in shell.get_script_integrations().unwrap_or_default() {
            if let Err(err) = script_integration.install().await {
                error!(%err, "Failed installing shell integration {}", script_integration.describe());
            }
        }

        for shell_integration in shell.get_shell_integrations(env).unwrap_or_default() {
            if let Err(err) = shell_integration.migrate().await {
                error!(%err, "Failed installing shell integration {}", shell_integration.describe());
            }
        }
    }

    Ok(())
}

fn should_run_install_script() -> bool {
    let current_version = current_version();
    let previous_version = match previous_version() {
        Some(previous_version) => previous_version,
        None => return true,
    };

    !is_cargo_debug_build() && current_version > previous_version
}

/// The current version of the desktop app
fn current_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap()
}

/// The previous version of the desktop app stored in local state
fn previous_version() -> Option<Version> {
    fig_settings::state::get_string(PREVIOUS_VERSION_KEY)
        .ok()
        .flatten()
        .and_then(|ref v| Version::parse(v).ok())
}

fn set_previous_version(version: Version) -> anyhow::Result<()> {
    fig_settings::state::set_value(PREVIOUS_VERSION_KEY, version.to_string())?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_current_version() {
        current_version();
    }

    #[tokio::test]
    async fn test_symlink() {
        use tempfile::tempdir;

        let tmp_dir = tempdir().unwrap();
        let tmp_dir = tmp_dir.path();

        // folders
        let src_dir_1 = tmp_dir.join("dir_1");
        let src_dir_2 = tmp_dir.join("dir_2");
        let dst_dir = tmp_dir.join("dst");

        std::fs::create_dir_all(&src_dir_1).unwrap();
        std::fs::create_dir_all(&src_dir_2).unwrap();

        // Check that a new symlink is created
        assert!(!dst_dir.exists());
        symlink(&src_dir_1, &dst_dir).await.unwrap();
        assert!(dst_dir.exists());
        assert_eq!(dst_dir.read_link().unwrap(), src_dir_1);

        // Check that the symlink is updated
        symlink(&src_dir_2, &dst_dir).await.unwrap();
        assert!(dst_dir.exists());
        assert_eq!(dst_dir.read_link().unwrap(), src_dir_2);

        // files
        let src_file_1 = src_dir_1.join("file_1");
        let src_file_2 = src_dir_2.join("file_2");
        let dst_file = dst_dir.join("file");

        std::fs::write(&src_file_1, "content 1").unwrap();
        std::fs::write(&src_file_2, "content 2").unwrap();

        // Check that a new symlink is created
        assert!(!dst_file.exists());
        symlink(&src_file_1, &dst_file).await.unwrap();
        assert!(dst_file.exists());
        assert_eq!(std::fs::read_to_string(&dst_file).unwrap(), "content 1");

        // Check that the symlink is updated
        symlink(&src_file_2, &dst_file).await.unwrap();
        assert!(dst_file.exists());
        assert_eq!(std::fs::read_to_string(&dst_file).unwrap(), "content 2");
    }
}
