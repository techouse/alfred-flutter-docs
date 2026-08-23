use super::*;
use crate::models::{EnclosedBy, SearchResult};
use anyhow::Result;

fn result(enclosed_by: Option<EnclosedBy>) -> SearchResult {
    SearchResult {
        object_id: "object-1".into(),
        name: "A &amp;".into(),
        qualified_name: "widgets.Container".into(),
        href: "widgets/Container-class.html".into(),
        type_name: "class".into(),
        enclosed_by,
    }
}

#[test]
fn renders_flutter_result_metadata_without_decoding_or_truncating() -> Result<()> {
    let item = items_from_results(&[result(Some(EnclosedBy {
        name: Some("widgets".into()),
    }))])?
    .remove(0);
    assert_eq!(item.title(), "A &amp; class");
    assert_eq!(item.subtitle(), Some("from widgets"));
    assert_eq!(
        item.arg(),
        Some("https://api.flutter.dev/flutter/widgets/Container-class.html")
    );
    assert_eq!(item.quick_look_url(), item.arg());
    assert_eq!(item.uid(), Some("object-1"));
    assert_eq!(item.text().map(|text| text.copy()), item.arg());
    assert_eq!(
        item.text().and_then(|text| text.large_type()),
        Some("widgets.Container")
    );
    assert_eq!(item.icon().map(|icon| icon.path()), Some("icon.png"));
    assert!(item.valid());
    Ok(())
}

#[test]
fn enclosure_without_name_preserves_from_null_and_null_enclosure_is_empty() -> Result<()> {
    let items = items_from_results(&[result(Some(EnclosedBy { name: None })), result(None)])?;
    assert_eq!(items[0].subtitle(), Some("from null"));
    assert_eq!(items[1].subtitle(), Some(""));
    Ok(())
}

#[test]
fn fallback_and_placeholder_keep_flutter_text() -> Result<()> {
    let fallback = google_fallback_item("request body")?;
    assert_eq!(
        fallback.arg(),
        Some("https://www.google.com/search?q=flutter+request+body")
    );
    assert!(fallback.valid());
    let placeholder = placeholder_item();
    assert_eq!(placeholder.title(), "Search the Flutter docs...");
    assert!(!placeholder.valid());
    Ok(())
}
