mod common;

use common::*;

#[test]
fn local_state_all() -> Result<()> {
    cli()
        .args(["_", "local-state", "all", "-f", "json"])
        .assert()
        .stdout(is_json())
        .success();
    Ok(())
}

#[test]
fn local_state_get() -> Result<()> {
    cli()
        .args(["_", "local-state", "test-value", "-f", "json"])
        .assert()
        .stdout(is_json())
        .success();
    Ok(())
}

#[test]
fn get_shell() -> Result<()> {
    cli().args(["_", "get-shell"]).assert().success();
    Ok(())
}

#[test]
fn should_figterm_launch_code_success() -> Result<()> {
    cli()
        .args(["_", "should-figterm-launch"])
        .env("AUTO_FORCE_FIGTERM_LAUNCH", "1")
        .assert()
        .success();
    Ok(())
}

#[test]
fn should_figterm_launch_code_failure() -> Result<()> {
    cli().args(["_", "should-figterm-launch"]).assert().failure();
    Ok(())
}
