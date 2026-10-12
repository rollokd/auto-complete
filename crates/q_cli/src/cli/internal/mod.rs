pub mod local_state;
pub mod should_figterm_launch;

use std::io::{
    Write,
    stdout,
};
use std::path::PathBuf;
use std::process::ExitCode;

use anstream::println;
use clap::Subcommand;
use eyre::Result;
use fig_integrations::input_method::InputMethod;
use fig_ipc::SendMessage;
use fig_proto::figterm::figterm_request_message::Request as FigtermRequest;
use fig_proto::figterm::{
    FigtermRequestMessage,
    UpdateShellContextRequest,
};
use fig_proto::local::EnvironmentVariable;
use fig_proto::util::get_shell;
use fig_util::directories::figterm_socket_path;
use fig_util::env_var::AUTOTERM_SESSION_ID;
use tracing::error;

#[derive(Debug, PartialEq, Eq, Subcommand)]
#[command(hide = true, alias = "_")]
pub enum InternalSubcommand {
    /// Command that is run during the PreCmd section of
    /// the shell integrations.
    PreCmd {
        #[arg(long, allow_hyphen_values = true)]
        alias: Option<String>,
    },
    /// Change the local-state file
    LocalState(local_state::LocalStateArgs),
    GetShell,
    /// Detects if Figterm should be launched
    ///
    /// Exit code:
    /// - 0 execute figterm
    /// - 1 dont execute figterm
    /// - 2 fallback to AUTO_TERM env
    ShouldFigtermLaunch,
    AttemptToFinishInputMethodInstallation {
        bundle_path: Option<PathBuf>,
    },
}

impl InternalSubcommand {
    pub async fn execute(self) -> Result<ExitCode> {
        match self {
            InternalSubcommand::PreCmd { alias } => Ok(pre_cmd(alias).await),
            InternalSubcommand::LocalState(local_state) => {
                local_state.execute().await?;
                Ok(ExitCode::SUCCESS)
            },
            InternalSubcommand::GetShell => match get_shell() {
                Ok(shell) => {
                    if write!(stdout(), "{shell}").is_ok() {
                        return Ok(ExitCode::SUCCESS);
                    }
                    Ok(ExitCode::FAILURE)
                },
                Err(_) => Ok(ExitCode::FAILURE),
            },
            InternalSubcommand::ShouldFigtermLaunch => {
                Ok(should_figterm_launch::should_figterm_launch(&fig_os_shim::Context::new()))
            },
            InternalSubcommand::AttemptToFinishInputMethodInstallation { bundle_path } => {
                match InputMethod::finish_input_method_installation(bundle_path) {
                    Ok(_) => Ok(ExitCode::SUCCESS),
                    Err(err) => {
                        println!(
                            "{}",
                            serde_json::to_string(&err).expect("InputMethodError should be serializable")
                        );
                        Ok(ExitCode::FAILURE)
                    },
                }
            },
        }
    }
}

pub async fn pre_cmd(alias: Option<String>) -> ExitCode {
    let Ok(session_id) = std::env::var(AUTOTERM_SESSION_ID) else {
        return ExitCode::FAILURE;
    };

    match figterm_socket_path(&session_id) {
        Ok(figterm_path) => match fig_ipc::socket_connect(figterm_path).await {
            Ok(mut figterm_stream) => {
                let message = FigtermRequestMessage {
                    request: Some(FigtermRequest::UpdateShellContext(UpdateShellContextRequest {
                        update_environment_variables: true,
                        environment_variables: std::env::vars()
                            .map(|(key, value)| EnvironmentVariable {
                                key,
                                value: Some(value),
                            })
                            .collect(),
                        update_alias: true,
                        alias,
                    })),
                };
                if let Err(err) = figterm_stream.send_message(message).await {
                    error!(%err, %session_id, "Failed to send UpdateShellContext to Figterm");
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            },
            Err(err) => {
                error!(%err, %session_id, "Failed to connect to Figterm socket");
                ExitCode::FAILURE
            },
        },
        Err(err) => {
            error!(%err, %session_id, "Failed to get Figterm socket path");
            ExitCode::FAILURE
        },
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use fig_util::CLI_BINARY_NAME;

    use super::*;

    #[derive(Debug, Parser, PartialEq, Eq)]
    pub struct MockCli {
        #[command(subcommand)]
        pub subcommand: InternalSubcommand,
    }

    #[test]
    fn parse_pre_cmd() {
        assert_eq!(MockCli::parse_from(["_", "pre-cmd"]), MockCli {
            subcommand: InternalSubcommand::PreCmd { alias: None }
        });

        let alias = format!("a='{CLI_BINARY_NAME} a'\nrd=rmdir");
        assert_eq!(MockCli::parse_from(["_", "pre-cmd", "--alias", &alias]), MockCli {
            subcommand: InternalSubcommand::PreCmd { alias: Some(alias) }
        });

        let hyphen_alias = "-='cd -'\n...=../..\nga='git add'";
        assert_eq!(
            MockCli::parse_from(["_", "pre-cmd", "--alias", hyphen_alias]),
            MockCli {
                subcommand: InternalSubcommand::PreCmd {
                    alias: Some(hyphen_alias.to_owned())
                }
            }
        );
    }
}
