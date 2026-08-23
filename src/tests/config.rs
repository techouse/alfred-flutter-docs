use std::env::VarError;

use super::*;

#[test]
fn runtime_dotenv_embedded_precedence_and_empty_values() -> Result<()> {
    assert_eq!(
        configuration_value(
            "SETTING",
            Ok("runtime".into()),
            Some("dotenv"),
            Some("embedded")
        )?,
        "runtime"
    );
    assert_eq!(
        configuration_value(
            "SETTING",
            Err(VarError::NotPresent),
            Some("dotenv"),
            Some("embedded")
        )?,
        "dotenv"
    );
    assert_eq!(
        configuration_value("SETTING", Err(VarError::NotPresent), None, Some("embedded"))?,
        "embedded"
    );
    for (runtime, dotenv, embedded) in [
        (Ok(String::new()), None, Some("x")),
        (Err(VarError::NotPresent), Some(""), Some("x")),
        (Err(VarError::NotPresent), None, Some("")),
    ] {
        assert_eq!(
            configuration_value("SETTING", runtime, dotenv, embedded)
                .expect_err("empty value must fail")
                .to_string(),
            "SETTING must not be empty"
        );
    }
    Ok(())
}

#[test]
fn complete_runtime_configuration_defers_malformed_dotenv() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join(".env");
    std::fs::write(&path, "BROKEN=\"unterminated\n")?;
    let config = algolia_search_config_from(
        Ok("app".into()),
        Ok("key".into()),
        Ok("index".into()),
        &path,
    )?;
    assert_eq!(config.index_name, "index");
    Ok(())
}

#[test]
fn missing_runtime_configuration_reads_only_explicit_dotenv() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join(".env");
    std::fs::write(&path, "ALGOLIA_SEARCH_INDEX=index\n")?;
    let config = algolia_search_config_from(
        Ok("app".into()),
        Ok("key".into()),
        Err(VarError::NotPresent),
        &path,
    )?;
    assert_eq!(config.index_name, "index");
    Ok(())
}

#[test]
fn dotenv_parse_errors_include_the_explicit_path() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("broken.env");
    std::fs::write(&path, "BROKEN=\"unterminated\n")?;
    let error = load_dotenv(&path).expect_err("malformed dotenv must fail");
    assert!(error.to_string().contains(&path.display().to_string()));
    Ok(())
}
