use std::cell::Cell;

use alfred_workflow_rs::{FileCache, Workflow};
use anyhow::anyhow;

use super::*;

fn settings() -> WorkflowSettings {
    WorkflowSettings {
        use_alfred_cache: false,
        use_file_cache: false,
        cache_ttl: Some(86_400),
        file_cache_max_entries: Some(1_280),
    }
}

#[test]
fn automatic_cache_wins_over_file_cache() {
    let mut workflow = Workflow::new();
    let mut configuration = settings();
    configuration.use_alfred_cache = true;
    configuration.use_file_cache = true;
    configure_cache(&mut workflow, "container", &configuration);
    assert!(workflow.use_automatic_cache());
    assert!(workflow.cache_key().is_none());
}

#[test]
fn empty_query_adds_placeholder_without_file_cache_or_search() -> Result<()> {
    let calls = Cell::new(0);
    let mut workflow = Workflow::with_file_cache(FileCache::with_path(tempfile::tempdir()?.path()));
    let cli = Cli::default();
    populate_workflow_with(&mut workflow, &cli, &settings(), |_| {
        calls.set(calls.get() + 1);
        Ok(Vec::new())
    })?;
    assert_eq!(calls.get(), 0);
    assert!(workflow.cache_key().is_none());
    assert_eq!(
        workflow.get_items()?.items()[0].title(),
        "Search the Flutter docs..."
    );
    Ok(())
}

#[test]
fn runtime_error_clears_file_and_automatic_cache_metadata() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut workflow = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    let mut configuration = settings();
    configuration.use_file_cache = true;
    let cli = Cli {
        query: "container".into(),
        ..Cli::default()
    };
    let error = populate_workflow_with(&mut workflow, &cli, &configuration, |_| {
        Err(anyhow!("transient failure"))
    })
    .expect_err("search must fail");
    replace_items_with_runtime_error(&mut workflow, &error)?;
    assert!(workflow.cache_key().is_none());
    assert!(!workflow.use_automatic_cache());
    let json: serde_json::Value = serde_json::from_str(&workflow.to_json_string()?)?;
    assert!(json.get("cache").is_none());
    assert_eq!(
        workflow.get_items()?.items()[0].title(),
        "transient failure"
    );
    Ok(())
}

#[test]
fn file_cache_hit_bypasses_search() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let mut cached = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    cached.set_cache_key(Some("container"));
    let expected = google_fallback_item("container")?;
    cached.add_item(expected.clone())?;

    let mut workflow = Workflow::with_file_cache(FileCache::with_path(directory.path()));
    let mut configuration = settings();
    configuration.use_file_cache = true;
    let calls = Cell::new(0);
    let cli = Cli {
        query: "container".into(),
        ..Cli::default()
    };
    populate_workflow_with(&mut workflow, &cli, &configuration, |_| {
        calls.set(calls.get() + 1);
        Ok(Vec::new())
    })?;
    assert_eq!(calls.get(), 0);
    assert_eq!(workflow.get_items()?.items(), &[expected]);
    Ok(())
}
