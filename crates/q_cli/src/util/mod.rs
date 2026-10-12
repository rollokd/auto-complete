pub mod desktop;

use std::ffi::OsStr;
use std::process::{
    Command,
    ExitCode,
};
use std::time::Duration;

use anstream::println;
use crossterm::style::Stylize;
use desktop::desktop_app_running;
use dialoguer::theme::ColorfulTheme;
use eyre::Result;
use fig_ipc::local::quit_command;
use fig_util::consts::APP_BUNDLE_ID;
use fig_util::{
    CLI_BINARY_NAME,
    PRODUCT_NAME,
};
use regex::Regex;

pub fn app_path_from_bundle_id(bundle_id: impl AsRef<OsStr>) -> Option<String> {
    let installed_apps = std::process::Command::new("mdfind")
        .arg("kMDItemCFBundleIdentifier")
        .arg("=")
        .arg(bundle_id)
        .output()
        .ok()?;

    let path = String::from_utf8_lossy(&installed_apps.stdout);
    Some(path.trim().split('\n').next()?.into())
}

pub async fn quit_fig(verbose: bool) -> Result<ExitCode> {
    if !desktop_app_running() {
        if verbose {
            println!("{PRODUCT_NAME} app is not running");
        }
        return Ok(ExitCode::SUCCESS);
    }

    if verbose {
        println!("Quitting {PRODUCT_NAME} app");
    }

    if quit_command().await.is_err() {
        tokio::time::sleep(Duration::from_millis(500)).await;
        let second_try = quit_command().await;
        if second_try.is_err() {
            if let Ok(info) = get_app_info() {
                let pid = Regex::new(r"pid = (\S+)")
                    .unwrap()
                    .captures(&info)
                    .and_then(|c| c.get(1));
                if let Some(pid) = pid {
                    let success = Command::new("kill")
                        .arg("-KILL")
                        .arg(pid.as_str())
                        .status()
                        .map(|res| res.success());
                    if let Ok(true) = success {
                        return Ok(ExitCode::SUCCESS);
                    }
                }
            }
            if verbose {
                println!("Unable to quit {PRODUCT_NAME} app");
            }

            second_try?;
        }
    }

    Ok(ExitCode::SUCCESS)
}

pub fn app_not_running_message() -> String {
    format!(
        "\n{}\n{PRODUCT_NAME} app might not be running, to launch {PRODUCT_NAME} run: {}\n",
        format!("Unable to connect to {PRODUCT_NAME} app").bold(),
        format!("{CLI_BINARY_NAME} launch").magenta()
    )
}

pub fn get_app_info() -> Result<String> {
    let output = Command::new("lsappinfo")
        .args(["info", "-app", APP_BUNDLE_ID])
        .output()?;
    let result = String::from_utf8(output.stdout)?;
    Ok(result.trim().into())
}

pub fn dialoguer_theme() -> ColorfulTheme {
    ColorfulTheme {
        prompt_prefix: dialoguer::console::style("?".into()).for_stderr().magenta(),
        ..ColorfulTheme::default()
    }
}
