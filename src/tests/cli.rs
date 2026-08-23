use super::*;

#[test]
fn parses_attached_and_collapsed_forms() -> Result<()> {
    let cli = Cli::parse(["-vuqContainer".to_owned()])?;
    assert!(cli.verbose);
    assert!(cli.update);
    assert_eq!(cli.query, "Container");

    let cli = Cli::parse(["-vq".to_owned(), "Container".to_owned()])?;
    assert_eq!(cli.query, "Container");
    assert!(cli.verbose);
    Ok(())
}

#[test]
fn parses_long_equals_and_unrecognized_dash_query() -> Result<()> {
    assert_eq!(Cli::parse(["--query=--force".to_owned()])?.query, "--force");
    assert_eq!(
        Cli::parse(["-q".to_owned(), "--force".to_owned()])?.query,
        "--force"
    );
    Ok(())
}

#[test]
fn rejects_options_and_valid_clusters_as_separated_query_values() {
    for value in [
        "-q",
        "--query",
        "--query=x",
        "-v",
        "--verbose",
        "-u",
        "--update",
        "-vu",
        "-vq",
    ] {
        let option = if value.starts_with("--") {
            "--query"
        } else {
            "-q"
        };
        let error = Cli::parse([option.to_owned(), value.to_owned()])
            .expect_err("option must not be consumed as query");
        assert_eq!(error.to_string(), format!("{option} requires a value"));
    }
}

#[test]
fn parses_query_bearing_cluster_and_normalizes_whitespace() -> Result<()> {
    let cli = Cli::parse(["-vuqFlutter".to_owned()])?;
    assert_eq!(cli.normalized_query(), "flutter");

    let cli = Cli {
        query: "  Container   Widget ".to_owned(),
        ..Cli::default()
    };
    assert_eq!(cli.normalized_query(), "container widget");
    Ok(())
}

#[test]
fn rejects_unknown_arguments_but_accepts_flags_in_any_order() -> Result<()> {
    assert!(Cli::parse(["-uv".to_owned()])?.update);
    let error = Cli::parse(["-vx".to_owned()]).expect_err("unknown cluster must fail");
    assert_eq!(error.to_string(), "unknown argument: -vx");
    Ok(())
}
