use alfred_workflow_rs::{Icon, Item, ItemText};
use anyhow::Result;
use url::Url;

use crate::models::SearchResult;

/// Builds the placeholder shown before the user enters a search query.
pub fn placeholder_item() -> Item {
    Item::new("Search the Flutter docs...").set_icon(Icon::new("icon.png"))
}

/// Converts ranked Flutter search results into Alfred items in provider order.
pub fn items_from_results(results: &[SearchResult]) -> Result<Vec<Item>> {
    results.iter().map(item_from_result).collect()
}

/// Builds the Google fallback shown when Algolia returns no hits.
pub fn google_fallback_item(query: &str) -> Result<Item> {
    let url = Url::parse_with_params(
        "https://www.google.com/search",
        [("q", format!("flutter {query}"))],
    )?;

    Ok(Item::builder("No matching answers found")
        .subtitle("Shall I try and search Google?")
        .arg(url.as_str())
        .text(ItemText::new(url.as_str()))
        .quick_look_url(url.as_str())
        .icon(Icon::new("google.png"))
        .valid(true)
        .build()?)
}

fn item_from_result(result: &SearchResult) -> Result<Item> {
    let url = format!("https://api.flutter.dev/flutter/{}", result.href);
    let subtitle = match &result.enclosed_by {
        Some(enclosed_by) => format!("from {}", enclosed_by.name.as_deref().unwrap_or("null")),
        None => String::new(),
    };

    Ok(
        Item::builder(format!("{} {}", result.name, result.type_name))
            .uid(&result.object_id)
            .subtitle(subtitle)
            .arg(&url)
            .text(ItemText::new(&url).with_large_type(&result.qualified_name))
            .quick_look_url(&url)
            .icon(Icon::new("icon.png"))
            .valid(true)
            .build()?,
    )
}

#[cfg(test)]
#[path = "tests/app.rs"]
mod tests;
