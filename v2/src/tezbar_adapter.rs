//! Compiled-in adapter for Tezbar beta fd300e276356587726566026f072563eb9c34cb1.
//! Include unchanged as a sibling of v2_sdk; link tezbar-markdown-to-jira by path.
use crate::native_core::common::{CoreContext, CoreResult};
use serde_json::{json, Value};

pub fn commands() -> &'static [&'static str] {
    &["markdown-to-jira"]
}

fn acknowledged(value: &Value) -> bool {
    value.is_object()
        && value.get("ok") == Some(&Value::Bool(true))
        && value.get("error").is_none_or(Value::is_null)
        && value.get("success").is_none_or(|v| v == &Value::Bool(true))
}

pub async fn handle(
    ctx: &CoreContext,
    command: &str,
    operation: &str,
    _payload: &Value,
    _state: &mut Value,
) -> CoreResult {
    if !commands().contains(&command) {
        return Err("Unknown Markdown to Jira command".into());
    }
    match operation {
        "extension:dispose-session" | "extension:refresh-session" => {
            return Ok(super::v2_sdk::unchanged())
        }
        "extension:run-command" => {}
        _ => return Err("Unsupported Markdown to Jira operation".into()),
    }
    // Never forward host errors: OS messages may contain the selected user text.
    let selection = ctx
        .request("core:platform:selected-text", vec![])
        .await
        .map_err(|_| "Failed to convert Markdown to Jira markup: selected-text request failed")?;
    let selection = selection.as_str().ok_or(
        "Failed to convert Markdown to Jira markup: selected-text response must be a string",
    )?;
    let result = tezbar_markdown_to_jira::to_jira(selection)
        .map_err(|error| format!("Failed to convert Markdown to Jira markup: {error}"))?;
    let paste_error = "Failed to convert Markdown to Jira markup: paste was not acknowledged. Clipboard, focus or pasted text may already have changed. No retry was performed; relaunching may paste again.";
    let response = ctx
        .request("core:platform:paste-text", vec![json!(result)])
        .await
        .map_err(|_| paste_error)?;
    if !acknowledged(&response) {
        return Err(paste_error.into());
    }
    // Keep the original explicit copy AFTER paste (the paste route also writes).
    let copy_error = "Failed to convert Markdown to Jira markup: final clipboard copy was not acknowledged after paste. The paste may already have happened. No retry was performed; relaunching may paste again.";
    let response = super::v2_sdk::copy(ctx, &result)
        .await
        .map_err(|_| copy_error)?;
    if !acknowledged(&response) {
        return Err(copy_error.into());
    }
    Ok(super::v2_sdk::no_view(
        "Successfully converted Markdown to Jira markup!",
    ))
}
