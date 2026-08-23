use std::process::Command;

use anyhow::Result;

fn workflow_binary() -> &'static str {
    env!("CARGO_BIN_EXE_alfred_flutter_docs")
}

#[test]
fn parse_errors_keep_exit_code_two_and_valid_json_stdout() -> Result<()> {
    let output = Command::new(workflow_binary())
        .args(["--unknown"])
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert!(
        json["items"][0]["title"]
            .as_str()
            .unwrap_or_default()
            .contains("unknown argument")
    );
    Ok(())
}

#[test]
fn runtime_errors_do_not_emit_automatic_cache_metadata() -> Result<()> {
    let output = Command::new(workflow_binary())
        .args(["-q", "Container"])
        .env("ALGOLIA_APPLICATION_ID", "")
        .env("ALGOLIA_SEARCH_ONLY_API_KEY", "runtime-key")
        .env("ALGOLIA_SEARCH_INDEX", "runtime-index")
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    assert!(json.get("cache").is_none());
    Ok(())
}
