use fig_integrations::shell::ShellExt;
use fig_os_shim::{
    ContextArcProvider,
    ContextProvider,
    EnvProvider,
};
use fig_proto::fig::{
    OnboardingAction,
    OnboardingRequest,
};
use fig_settings::{
    SettingsProvider,
    StateProvider,
};
use fig_util::{
    CLI_BINARY_NAME,
    Shell,
};
use tao::event_loop::ControlFlow;
use tokio::process::Command;
use tracing::error;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::EventLoopProxy;
use crate::event::Event;

pub async fn post_login() {
    fig_settings::state::set_value("desktop.completedOnboarding", true).ok();
}

pub async fn onboarding<Ctx>(request: OnboardingRequest, proxy: &EventLoopProxy, ctx: &Ctx) -> RequestResult
where
    Ctx: SettingsProvider + StateProvider + ContextProvider + ContextArcProvider + Send + Sync,
{
    match request.action() {
        OnboardingAction::InstallationScript => {
            let mut errs: Vec<String> = vec![];
            for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
                match shell.get_shell_integrations(ctx.env()) {
                    Ok(integrations) => {
                        for integration in integrations {
                            if let Err(err) = integration.install().await {
                                errs.push(format!("{integration}: {err}"));
                            }
                        }
                    },
                    Err(err) => {
                        errs.push(format!("{shell}: {err}"));
                    },
                }
            }

            match &errs[..] {
                [] => RequestResult::success(),
                errs => RequestResult::error(errs.join("\n")),
            }
        },
        OnboardingAction::Uninstall => {
            let Some(cli) = macos_utils::bundle::get_bundle_path_for_executable(CLI_BINARY_NAME) else {
                return RequestResult::error(format!("Failed to find {CLI_BINARY_NAME} in bundle"));
            };

            let result = match tokio::process::Command::new(cli)
                .args(["uninstall", "--no-confirm"])
                .status()
                .await
            {
                Ok(status) if status.success() => RequestResult::success(),
                Ok(status) => RequestResult::error(format!("Uninstall failed: {status}")),
                Err(err) => RequestResult::error(err.to_string()),
            };

            proxy.send_event(Event::ControlFlow(ControlFlow::Exit)).ok();
            result
        },
        OnboardingAction::FinishOnboarding => {
            post_login().await;
            RequestResult::success()
        },
        OnboardingAction::LaunchShellOnboarding => {
            fig_settings::state::set_value("user.onboarding", false).ok();

            let home = fig_util::directories::home_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));

            if let Err(err) = Command::new("open").args(["-b", "com.apple.Terminal"]).arg(home).spawn() {
                error!(%err, "Failed to open onboarding");
                return RequestResult::error("Failed to open onboarding");
            }

            RequestResult::success()
        },
        OnboardingAction::PromptForAccessibilityPermission => {
            use crate::local_ipc::{
                LocalResponse,
                commands,
            };
            let res = commands::prompt_for_accessibility_permission(ctx)
                .await
                .unwrap_or_else(|e| e);
            match res {
                LocalResponse::Success(_) => RequestResult::success(),
                LocalResponse::Error {
                    message: Some(message), ..
                } => RequestResult::error(message),
                _ => RequestResult::error("Failed to prompt for accessibility permissions"),
            }
        },
        OnboardingAction::PostLogin => {
            post_login().await;
            RequestResult::success()
        },
        OnboardingAction::CloseAccessibilityPromptWindow
        | OnboardingAction::RequestRestart
        | OnboardingAction::CloseInputMethodPromptWindow => RequestResult::error("Unimplemented"),
    }
}
