//! The served document names each schema once. utoipa keeps one component
//! per name: a hand-written schema named like a generated one replaces it
//! (`crate::openapi::ApiDoc` wins the merge), and every generated operation
//! then documents the hand-written shape. These tests fail when that
//! happens, so a column added to a table shows in its operations' schemas.

use axum_api::features::GeneratedApiDoc;
use axum_api::openapi::{ApiDoc, document};
use serde_json::Value;
use utoipa::OpenApi;

/// The component schemas of `doc`, by name.
fn schemas(doc: &utoipa::openapi::OpenApi) -> Result<serde_json::Map<String, Value>, String> {
    let doc = serde_json::to_value(doc).map_err(|error| error.to_string())?;
    Ok(doc
        .pointer("/components/schemas")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default())
}

/// Each schema name the hand-written and the generated documents both
/// define must define the same schema; otherwise the merge drops one.
#[test]
fn hand_written_schemas_never_replace_generated_ones_of_the_same_name() -> Result<(), String> {
    let (hand_written, generated) = (
        schemas(&ApiDoc::openapi())?,
        schemas(&GeneratedApiDoc::openapi())?,
    );
    let replaced: Vec<&String> = hand_written
        .iter()
        .filter(|(name, schema)| generated.get(*name).is_some_and(|other| other != *schema))
        .map(|(name, _)| name)
        .collect();
    assert!(
        replaced.is_empty(),
        "hand-written schemas replace the generated ones of the same name (rename yours): {replaced:?}"
    );
    Ok(())
}

/// `GET /api/v1/users/{id}` answers the `users` feature's `UserResponse`,
/// the schema the list's items have.
#[test]
fn a_user_by_id_and_the_list_items_share_the_users_schema() -> Result<(), String> {
    let doc = serde_json::to_value(document()).map_err(|error| error.to_string())?;
    let body = |path: &str| {
        doc.pointer(&format!(
            "/paths/{}/get/responses/200/content/application~1json/schema",
            path.replace('/', "~1")
        ))
        .cloned()
        .ok_or(format!("GET {path} documents no JSON answer"))
    };
    let resolve = |schema: Value| match schema.get("$ref").and_then(Value::as_str) {
        Some(reference) => doc
            .pointer(reference.trim_start_matches('#'))
            .cloned()
            .ok_or(format!("{reference} is not defined")),
        None => Ok(schema),
    };
    assert_eq!(
        body("/api/v1/users/{id}")?,
        serde_json::json!({"$ref": "#/components/schemas/UserResponse"})
    );
    let page = resolve(body("/api/v1/users")?)?;
    let items = page
        .pointer("/properties/items/items")
        .cloned()
        .ok_or("the list's page has no items")?;
    assert_eq!(
        resolve(items)?,
        resolve(body("/api/v1/users/{id}")?)?,
        "the list's items and `GET /api/v1/users/{{id}}` document different users"
    );
    Ok(())
}
