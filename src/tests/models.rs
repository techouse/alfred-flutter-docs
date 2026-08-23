use anyhow::Result;
use serde_json::json;

use crate::models::{SearchResponse, SearchResult};

#[test]
fn deserializes_required_flutter_fields_and_nullable_enclosure() -> Result<()> {
    let response: SearchResponse = serde_json::from_value(json!({
        "hits": [{
            "objectID": "one",
            "name": "Container",
            "qualifiedName": "widgets.Container",
            "href": "widgets/Container-class.html",
            "type": "class",
            "enclosedBy": {"name": "widgets"}
        }, {
            "objectID": "two",
            "name": "Widget",
            "qualifiedName": "widgets.Widget",
            "href": "widgets/Widget-class.html",
            "type": "class",
            "enclosedBy": null
        }]
    }))?;
    assert_eq!(response.hits[0].object_id, "one");
    assert!(response.hits[1].enclosed_by.is_none());
    Ok(())
}

#[test]
fn required_fields_are_not_optional() {
    let value = json!({"objectID":"one", "name":"Container"});
    assert!(serde_json::from_value::<SearchResult>(value).is_err());
}
