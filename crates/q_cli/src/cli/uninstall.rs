use std::process::ExitCode;

use anstream::println;
use eyre::Result;
use fig_integrations::Integration;
use fig_integrations::input_method::{
    InputMethod,
    InputMethodError,
};
use fig_integrations::shell::ShellExt;
use fig_os_shim::Env;
use fig_util::consts::LAUNCH_AGENT_LABEL;
use fig_util::directories::home_local_bin;
use fig_util::launchd_plist::LaunchdPlist;
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
    PTY_BINARY_NAME,
    Shell,
};
use tracing::warn;

use crate::util::dialoguer_theme;

pub async fn uninstall_command(no_confirm: bool) -> Result<ExitCode> {
    if !no_confirm {
        let should_continue = dialoguer::Select::with_theme(&dialoguer_theme())
            .with_prompt(format!("Are you sure want to uninstall {PRODUCT_NAME}?"))
            .items(&["Yes", "No"])
            .default(0)
            .interact_opt()?;

        if should_continue != Some(0) {
            println!("Cancelled");
            return Ok(ExitCode::FAILURE);
        }
    }

    crate::util::quit_fig(false).await.ok();

    for shell in Shell::all() {
        for integration in shell.get_shell_integrations(&Env::new()).unwrap_or_default() {
            if let Err(err) = integration.uninstall().await {
                warn!(%err, "Failed to uninstall {}", integration.describe());
            }
        }
    }

    match InputMethod::default().uninstall().await {
        Ok(()) | Err(fig_integrations::Error::InputMethod(InputMethodError::CouldNotListInputSources)) => {},
        Err(err) => warn!(%err, "Failed to uninstall input method"),
    }

    if let Ok(path) = LaunchdPlist::new(LAUNCH_AGENT_LABEL).get_file_path() {
        if path.exists() {
            std::process::Command::new("launchctl").arg("unload").arg(&path).status().ok();
            std::fs::remove_file(&path).ok();
        }
    }

    if let Ok(local_bin) = home_local_bin() {
        let mut binaries = vec![CLI_BINARY_NAME.to_owned(), PTY_BINARY_NAME.to_owned()];
        binaries.extend(Shell::all().iter().map(|shell| format!("{shell} ({PTY_BINARY_NAME})")));
        for binary in binaries {
            let path = local_bin.join(binary);
            if path.symlink_metadata().is_ok() {
                std::fs::remove_file(&path).ok();
            }
        }
    }

    println!("{PRODUCT_NAME} has been uninstalled. You can now delete {PRODUCT_NAME}.app");
    Ok(ExitCode::SUCCESS)
}
