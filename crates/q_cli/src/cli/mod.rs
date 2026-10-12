//! CLI functionality

pub mod app;
mod completion;
mod init;
mod integrations;
pub mod internal;
mod settings;
mod theme;
mod uninstall;

use std::io::{
    Write as _,
    stdout,
};
use std::process::ExitCode;

use anstream::{
    eprintln,
    println,
};
use clap::{
    ArgAction,
    CommandFactory,
    Parser,
    Subcommand,
    ValueEnum,
};
use crossterm::style::Stylize;
use eyre::{
    Result,
    WrapErr,
};
use fig_ipc::local::open_ui_element;
use fig_log::{
    LogArgs,
    initialize_logging,
};
use fig_proto::local::UiElement;
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
    directories,
};
use serde::Serialize;
use tracing::{
    Level,
    debug,
};

use self::integrations::IntegrationsSubcommands;
use crate::util::desktop::{
    LaunchArgs,
    launch_fig_desktop,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    /// Outputs the results as markdown
    #[default]
    Plain,
    /// Outputs the results as JSON
    Json,
    /// Outputs the results as pretty print JSON
    JsonPretty,
}

impl OutputFormat {
    pub fn print<T, TFn, J, JFn>(&self, text_fn: TFn, json_fn: JFn)
    where
        T: std::fmt::Display,
        TFn: FnOnce() -> T,
        J: Serialize,
        JFn: FnOnce() -> J,
    {
        match self {
            OutputFormat::Plain => println!("{}", text_fn()),
            OutputFormat::Json => println!("{}", serde_json::to_string(&json_fn()).unwrap()),
            OutputFormat::JsonPretty => println!("{}", serde_json::to_string_pretty(&json_fn()).unwrap()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Processes {
    /// Desktop Process
    App,
}

/// The Auto CLI
#[deny(missing_docs)]
#[derive(Debug, PartialEq, Subcommand)]
pub enum CliRootCommands {
    /// Customize appearance & behavior
    #[command(alias("setting"))]
    Settings(settings::SettingsArgs),
    /// Manage system integrations
    #[command(subcommand, alias("integration"))]
    Integrations(IntegrationsSubcommands),
    /// Generate the dotfiles for the given shell
    Init(init::InitArgs),
    /// Get or set theme
    Theme(theme::ThemeArgs),
    /// Launch the desktop app
    Launch,
    /// Quit the desktop app
    Quit,
    /// Restart the desktop app
    Restart {
        /// The process to restart
        #[arg(value_enum, default_value_t = Processes::App, hide = true)]
        process: Processes,
    },
    /// Remove shell integrations, the input method, the launch agent and CLI symlinks
    Uninstall {
        /// Skip confirmation
        #[arg(long, short = 'y')]
        no_confirm: bool,
    },
    /// Generate CLI completion spec
    #[command(hide = true)]
    Completion(completion::CompletionArgs),
    /// Internal subcommands
    #[command(subcommand, hide = true)]
    Internal(internal::InternalSubcommand),
    /// Version
    #[command(hide = true)]
    Version,
}

const HELP_TEXT: &str = color_print::cstr! {"

<magenta,em>auto</magenta,em> (Auto CLI)

<magenta,em>Popular Subcommands</magenta,em>              <black!><em>Usage:</em> auto [subcommand]</black!>
╭────────────────────────────────────────────────────╮
│ <em>settings</em>     <black!>Customize appearance & behavior</black!>       │
│ <em>integrations</em> <black!>Manage system integrations</black!>            │
│ <em>restart</em>      <black!>Restart the app</black!>                       │
│ <em>quit</em>         <black!>Quit the app</black!>                          │
╰────────────────────────────────────────────────────╯

<black!>To see all subcommands, use:</black!>
 <black!>❯</black!> auto --help-all
ㅤ
"};

#[derive(Debug, Parser, PartialEq, Default)]
#[command(version, about, name = CLI_BINARY_NAME, help_template = HELP_TEXT)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: Option<CliRootCommands>,
    /// Increase logging verbosity
    #[arg(long, short = 'v', action = ArgAction::Count, global = true)]
    pub verbose: u8,
    /// Print help for all subcommands
    #[arg(long)]
    help_all: bool,
}

impl Cli {
    pub async fn execute(self) -> Result<ExitCode> {
        // Initialize our logger and keep around the guard so logging can perform as expected.
        let _log_guard = initialize_logging(LogArgs {
            log_level: match self.verbose > 0 {
                true => Some(
                    match self.verbose {
                        1 => Level::WARN,
                        2 => Level::INFO,
                        3 => Level::DEBUG,
                        _ => Level::TRACE,
                    }
                    .to_string(),
                ),
                false => None,
            },
            log_to_stdout: fig_os_shim::Env::new().q_log_stdout() || self.verbose > 0,
            log_file_path: match fig_log::get_log_level_max() >= Level::DEBUG {
                true => Some("cli.log".to_owned()),
                false => None,
            }
            .map(|name| directories::logs_dir().expect("home dir must be set").join(name)),
            delete_old_log_file: false,
        });

        debug!(command =? std::env::args().collect::<Vec<_>>(), "Command ran");

        if self.help_all {
            return self.print_help_all();
        }

        match self.subcommand {
            Some(subcommand) => match subcommand {
                CliRootCommands::Settings(settings_args) => settings_args.execute().await,
                CliRootCommands::Integrations(subcommand) => subcommand.execute().await,
                CliRootCommands::Init(args) => args.execute().await,
                CliRootCommands::Theme(theme_args) => theme_args.execute().await,
                CliRootCommands::Launch => {
                    launch_fig_desktop(LaunchArgs {
                        wait_for_socket: true,
                        open_dashboard: false,
                        verbose: true,
                    })?;
                    Ok(ExitCode::SUCCESS)
                },
                CliRootCommands::Quit => crate::util::quit_fig(true).await,
                CliRootCommands::Restart { .. } => {
                    app::restart_fig().await?;
                    Ok(ExitCode::SUCCESS)
                },
                CliRootCommands::Uninstall { no_confirm } => uninstall::uninstall_command(no_confirm).await,
                CliRootCommands::Completion(args) => args.execute(),
                CliRootCommands::Internal(internal_subcommand) => internal_subcommand.execute().await,
                CliRootCommands::Version => {
                    let _ = writeln!(stdout(), "{}", Self::command().render_version());
                    Ok(ExitCode::SUCCESS)
                },
            },
            None => {
                Self::command().print_help()?;
                Ok(ExitCode::SUCCESS)
            },
        }
    }

    #[allow(clippy::unused_self)]
    fn print_help_all(&self) -> Result<ExitCode> {
        let mut cmd = Self::command().help_template("{all-args}");
        eprintln!();
        eprintln!(
            "{}\n    {CLI_BINARY_NAME} [OPTIONS] [SUBCOMMAND]\n",
            "USAGE:".bold().underlined(),
        );
        cmd.print_long_help()?;
        Ok(ExitCode::SUCCESS)
    }
}

/// Launch the app and open the settings window at `route`
pub async fn open_dashboard(route: Option<String>) -> Result<ExitCode> {
    launch_fig_desktop(LaunchArgs {
        wait_for_socket: true,
        open_dashboard: false,
        verbose: true,
    })?;

    println!("Opening {PRODUCT_NAME} settings");

    open_ui_element(UiElement::MissionControl, route)
        .await
        .context("Failed to open settings")?;

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn debug_assert() {
        Cli::command().debug_assert();
    }

    macro_rules! assert_parse {
        (
            [ $($args:expr),+ ],
            $subcommand:expr
        ) => {
            assert_eq!(
                Cli::parse_from([CLI_BINARY_NAME, $($args),*]),
                Cli {
                    subcommand: Some($subcommand),
                    ..Default::default()
                }
            );
        };
    }

    #[test]
    fn test_flags() {
        assert_eq!(Cli::parse_from([CLI_BINARY_NAME, "-v"]), Cli {
            subcommand: None,
            verbose: 1,
            help_all: false,
        });

        assert_eq!(Cli::parse_from([CLI_BINARY_NAME, "--help-all"]), Cli {
            subcommand: None,
            verbose: 0,
            help_all: true,
        });
    }

    #[test]
    fn test_restart() {
        assert_parse!(["restart", "app"], CliRootCommands::Restart {
            process: Processes::App
        });
    }

    /// Must match how fig_integrations::input_method invokes the CLI
    #[cfg(target_os = "macos")]
    #[test]
    fn test_input_method_installation() {
        use internal::InternalSubcommand;
        assert_parse!(
            [
                "_",
                "attempt-to-finish-input-method-installation",
                "/path/to/bundle.app"
            ],
            CliRootCommands::Internal(InternalSubcommand::AttemptToFinishInputMethodInstallation {
                bundle_path: Some(std::path::PathBuf::from("/path/to/bundle.app"))
            })
        );
    }
}
