pub const APP_BUNDLE_ID: &str = "dev.rollo.auto";
pub const APP_BUNDLE_NAME: &str = "Auto.app";

/// Label of the LaunchAgent used to launch the app at login
pub const LAUNCH_AGENT_LABEL: &str = "dev.rollo.auto.launcher";

#[cfg(target_os = "macos")]
pub const APP_PROCESS_NAME: &str = "auto_desktop";
#[cfg(target_os = "linux")]
pub const APP_PROCESS_NAME: &str = "auto-desktop";

#[cfg(windows)]
pub const APP_PROCESS_NAME: &str = "auto_desktop.exe";

/// The name configured under `"package.productName"` in the tauri.conf.json file.
pub const TAURI_PRODUCT_NAME: &str = "auto_desktop";

pub const CLI_BINARY_NAME: &str = "auto";
pub const PTY_BINARY_NAME: &str = "autoterm";

pub const URL_SCHEMA: &str = "auto";

pub const PRODUCT_NAME: &str = "Auto";

pub const RUNTIME_DIR_NAME: &str = "autorun";

/// Data directory name used in paths like ~/.local/share/{DATA_DIR_NAME}
#[cfg(unix)]
pub const DATA_DIR_NAME: &str = "auto";
#[cfg(windows)]
pub const DATA_DIR_NAME: &str = "Auto";

/// Backup directory name
pub const BACKUP_DIR_NAME: &str = ".auto.dotfiles.bak";

/// Build time env vars
pub mod build {
    /// The target of the current build, e.g. "aarch64-unknown-linux-musl"
    pub const TARGET_TRIPLE: Option<&str> = option_env!("AUTO_BUILD_TARGET_TRIPLE");

    /// The variant of the current build
    pub const VARIANT: Option<&str> = option_env!("AUTO_BUILD_VARIANT");

    /// A git full sha hash of the current build
    pub const HASH: Option<&str> = option_env!("AUTO_BUILD_HASH");

    /// The datetime in rfc3339 format of the current build
    pub const DATETIME: Option<&str> = option_env!("AUTO_BUILD_DATETIME");

    /// If `fish` tests should be skipped
    pub const SKIP_FISH_TESTS: bool = option_env!("AUTO_BUILD_SKIP_FISH_TESTS").is_some();

    /// If `shellcheck` tests should be skipped
    pub const SKIP_SHELLCHECK_TESTS: bool = option_env!("AUTO_BUILD_SKIP_SHELLCHECK_TESTS").is_some();
}

/// macOS specific constants
pub mod macos {
    pub const BUNDLE_CONTENTS_MACOS_PATH: &str = "Contents/MacOS";
    pub const BUNDLE_CONTENTS_RESOURCE_PATH: &str = "Contents/Resources";
    pub const BUNDLE_CONTENTS_HELPERS_PATH: &str = "Contents/Helpers";
    pub const BUNDLE_CONTENTS_INFO_PLIST_PATH: &str = "Contents/Info.plist";
}

pub mod linux {
    /// Name of the package.
    pub const PACKAGE_NAME: &str = "auto";

    /// The wm_class used for the application windows.
    pub const DESKTOP_APP_WM_CLASS: &str = "Auto";
}

pub mod env_var {
    macro_rules! define_env_vars {
        ($($(#[$meta:meta])* $ident:ident = $name:expr),*) => {
            $(
                $(#[$meta])*
                pub const $ident: &str = $name;
            )*

            pub const ALL: &[&str] = &[$($ident),*];
        }
    }

    define_env_vars! {
        /// The UUID of the current parent autoterm instance
        AUTOTERM_SESSION_ID = "AUTOTERM_SESSION_ID",

        /// The current parent socket to connect to
        AUTO_PARENT = "AUTO_PARENT",

        /// Set the [`AUTO_PARENT`] parent socket to connect to
        AUTO_SET_PARENT = "AUTO_SET_PARENT",

        /// Guard for the [`AUTO_SET_PARENT`] check
        AUTO_SET_PARENT_CHECK = "AUTO_SET_PARENT_CHECK",

        /// Set if autoterm is running, contains the version
        AUTO_TERM = "AUTO_TERM",

        /// Sets the current log level
        AUTO_LOG_LEVEL = "AUTO_LOG_LEVEL",

        /// Overrides the ZDOTDIR environment variable
        AUTO_ZDOTDIR = "AUTO_ZDOTDIR",

        /// Indicates a process was launched by Auto
        PROCESS_LAUNCHED_BY_AUTO = "PROCESS_LAUNCHED_BY_AUTO",

        /// The shell to use in autoterm
        AUTO_SHELL = "AUTO_SHELL",

        /// Indicates the user is debugging the shell
        AUTO_DEBUG_SHELL = "AUTO_DEBUG_SHELL",

        /// Indicates the user is using zsh autosuggestions which disables Inline
        AUTO_USING_ZSH_AUTOSUGGESTIONS = "AUTO_USING_ZSH_AUTOSUGGESTIONS",

        /// Overrides the path to the bundle metadata released with certain desktop builds.
        AUTO_BUNDLE_METADATA_PATH = "AUTO_BUNDLE_METADATA_PATH"
    }
}

pub mod system_paths {
    /// System installation paths
    pub const APPLICATIONS_DIR: &str = "/Applications";
    pub const USR_LOCAL_BIN: &str = "/usr/local/bin";
    pub const USR_SHARE: &str = "/usr/share";
    pub const OPT_HOMEBREW_BIN: &str = "/opt/homebrew/bin";
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;
    use time::format_description::well_known::Rfc3339;

    use super::*;

    #[test]
    fn test_build_envs() {
        if let Some(build_variant) = build::VARIANT {
            println!("build_variant: {build_variant}");
            assert!(["full", "minimal"].contains(&&*build_variant.to_ascii_lowercase()));
        }

        if let Some(build_hash) = build::HASH {
            println!("build_hash: {build_hash}");
            assert!(!build_hash.is_empty());
        }

        if let Some(build_datetime) = build::DATETIME {
            println!("build_datetime: {build_datetime}");
            println!("{}", OffsetDateTime::parse(build_datetime, &Rfc3339).unwrap());
        }
    }
}
