use std::process::ExitCode;

use anstream::println;
use clap::Subcommand;
use crossterm::style::Stylize;
use eyre::Result;
use fig_integrations::Integration as _;
use fig_integrations::input_method::InputMethod;
use fig_integrations::shell::ShellExt;
use fig_os_shim::Env;
use fig_util::Shell;
use macos_utils::accessibility::{
    accessibility_is_enabled,
    open_accessibility,
};
use serde_json::json;
use tracing::debug;

use super::OutputFormat;

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum IntegrationsSubcommands {
    Install {
        /// Integration to install
        #[command(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[arg(long, short)]
        silent: bool,
    },
    Uninstall {
        /// Integration to uninstall
        #[command(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[arg(long, short)]
        silent: bool,
    },
    Reinstall {
        /// Integration to reinstall
        #[command(subcommand)]
        integration: Integration,
        /// Suppress status messages
        #[arg(long, short)]
        silent: bool,
    },
    Status {
        /// Integration to check status of
        #[command(subcommand)]
        integration: Integration,
        #[arg(long, short, value_enum, default_value_t)]
        format: OutputFormat,
    },
}

#[derive(Debug, Subcommand, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Integration {
    Dotfiles {
        /// Only install the integrations for a single shell
        #[arg(value_enum)]
        shell: Option<Shell>,
    },
    InputMethod,
    Accessibility,
    All,
}

const ALL: [Integration; 3] = [
    Integration::Dotfiles { shell: None },
    Integration::InputMethod,
    Integration::Accessibility,
];

impl IntegrationsSubcommands {
    pub async fn execute(self) -> Result<ExitCode> {
        match self {
            IntegrationsSubcommands::Install { integration, silent } => {
                for integration in expand(integration) {
                    install(integration, silent).await?;
                }
                Ok(ExitCode::SUCCESS)
            },
            IntegrationsSubcommands::Uninstall { integration, silent } => {
                for integration in expand(integration) {
                    uninstall(integration, silent).await?;
                }
                Ok(ExitCode::SUCCESS)
            },
            IntegrationsSubcommands::Reinstall { integration, silent } => {
                for integration in expand(integration) {
                    uninstall(integration, silent).await?;
                    install(integration, silent).await?;
                }
                Ok(ExitCode::SUCCESS)
            },
            IntegrationsSubcommands::Status { integration, format } => {
                for integration in expand(integration) {
                    status(integration, format).await?;
                }
                Ok(ExitCode::SUCCESS)
            },
        }
    }
}

fn expand(integration: Integration) -> Vec<Integration> {
    match integration {
        Integration::All => ALL.to_vec(),
        integration => vec![integration],
    }
}

fn shells(shell: Option<Shell>) -> Vec<Shell> {
    match shell {
        Some(shell) => vec![shell],
        None => Shell::supported().to_vec(),
    }
}

async fn install(integration: Integration, silent: bool) -> Result<()> {
    let mut installed = false;
    let mut status: Option<&str> = None;

    let result = match integration {
        Integration::All => Ok(()),
        Integration::Dotfiles { shell } => {
            let mut errs: Vec<String> = vec![];
            for shell in shells(shell) {
                match shell.get_shell_integrations(&Env::new()) {
                    Ok(integrations) => {
                        for integration in integrations {
                            match integration.is_installed().await {
                                Ok(_) => {
                                    debug!("Skipping {}", integration.describe());
                                },
                                Err(_) => {
                                    installed = true;
                                    if let Err(e) = integration.install().await {
                                        errs.push(format!(
                                            "{}: {}",
                                            integration.describe().bold(),
                                            e.verbose_message()
                                        ));
                                    }
                                },
                            }
                        }
                    },
                    Err(e) => {
                        errs.push(format!("{shell}: {e}"));
                    },
                }
            }

            if errs.is_empty() {
                Ok(())
            } else {
                Err(eyre::eyre!("\n\n{}", errs.join("\n\n")))
            }
        },
        Integration::InputMethod => {
            fig_settings::state::set_value("input-method.enabled", true).ok();
            InputMethod::default().install().await?;
            installed = true;
            status = Some("You must restart your terminal to finish installing the input method.");
            Ok(())
        },
        Integration::Accessibility => {
            if !accessibility_is_enabled() {
                open_accessibility();
                installed = true;
                status = Some("Grant accessibility permission in System Settings to finish.");
            }
            Ok(())
        },
    };

    if installed && result.is_ok() && !silent {
        println!("Installed!");

        if let Some(status) = status {
            println!("{status}");
        }
    }

    if !installed && !silent {
        println!("Already installed");
    }

    result
}

async fn uninstall(integration: Integration, silent: bool) -> Result<()> {
    let mut uninstalled = false;

    let result = match integration {
        Integration::All => Ok(()),
        Integration::Dotfiles { shell } => {
            let mut errs: Vec<String> = vec![];
            for shell in shells(shell) {
                match shell.get_shell_integrations(&Env::new()) {
                    Ok(integrations) => {
                        for integration in integrations {
                            match integration.is_installed().await {
                                Ok(_) => {
                                    uninstalled = true;
                                    if let Err(e) = integration.uninstall().await {
                                        errs.push(format!(
                                            "{}: {}",
                                            integration.describe().bold(),
                                            e.verbose_message()
                                        ));
                                    }
                                },
                                Err(_) => {
                                    debug!("Skipping {}", integration.describe());
                                },
                            }
                        }
                    },
                    Err(e) => {
                        errs.push(format!("{shell}: {e}"));
                    },
                }
            }

            if errs.is_empty() {
                Ok(())
            } else {
                Err(eyre::eyre!("\n\n{}", errs.join("\n\n")))
            }
        },
        Integration::InputMethod => {
            InputMethod::default().uninstall().await?;
            uninstalled = true;
            Ok(())
        },
        Integration::Accessibility => {
            if !silent {
                println!("Remove accessibility permission manually in System Settings > Privacy & Security");
            }
            return Ok(());
        },
    };

    if uninstalled && result.is_ok() && !silent {
        println!("Uninstalled!");
    }

    if !uninstalled && !silent {
        println!("Not installed");
    }

    result
}

async fn status(integration: Integration, format: OutputFormat) -> Result<()> {
    match integration {
        Integration::All => {},
        Integration::Dotfiles { shell } => {
            let mut all_integrations = vec![];
            let mut errors = vec![];

            for shell in shells(shell) {
                match shell.get_shell_integrations(&Env::new()) {
                    Ok(integrations) => {
                        for integration in integrations {
                            all_integrations.push((
                                integration.is_installed().await.is_ok(),
                                integration.describe(),
                                integration.get_shell(),
                                integration.file_name().to_owned(),
                            ));
                        }
                    },
                    Err(e) => {
                        errors.push((shell.to_string(), e.verbose_message()));
                    },
                }
            }

            format.print(
                || {
                    let mut s = String::new();
                    for (installed, describe, _, _) in &all_integrations {
                        s.push_str(&mark(*installed));
                        s.push_str(describe);
                        s.push('\n');
                    }

                    for (shell, error) in &errors {
                        s.push_str(&format!("{shell}: {error}\n"));
                    }

                    s.trim_end().to_owned()
                },
                || {
                    let integrations = all_integrations
                        .iter()
                        .map(|(installed, describe, shell, file_name)| {
                            json!({
                                "installed": installed,
                                "description": describe,
                                "shell": shell,
                                "file_name": file_name,
                            })
                        })
                        .collect::<Vec<_>>();

                    let errors = errors
                        .iter()
                        .map(|(shell, error)| {
                            json!({
                                "shell": shell,
                                "error": error,
                            })
                        })
                        .collect::<Vec<_>>();

                    json!({
                        "integrations": integrations,
                        "errors": errors,
                    })
                },
            );
        },
        Integration::InputMethod => {
            let installed = InputMethod::default().is_installed().await.is_ok();
            print_installed(format, "Input method", installed);
        },
        Integration::Accessibility => {
            print_installed(format, "Accessibility", accessibility_is_enabled());
        },
    }

    Ok(())
}

fn mark(installed: bool) -> String {
    if installed {
        "✔ ".green().to_string()
    } else {
        "✘ ".red().to_string()
    }
}

fn print_installed(format: OutputFormat, name: &str, installed: bool) {
    format.print(
        || format!("{}{name}", mark(installed)),
        || json!({ "name": name, "installed": installed }),
    );
}
