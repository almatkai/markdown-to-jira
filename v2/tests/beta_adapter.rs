// Only the Tauri event boundary and OS router are stubbed. CoreContext and SDK
// below are byte-exact files from beta fd300e2; the production adapter is unchanged.
extern crate self as tauri;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
#[derive(Clone)]
pub struct AppHandle;
pub trait Emitter {
    fn emit(&self, channel: &str, payload: Value) -> Result<(), String>;
}
impl Emitter for AppHandle {
    fn emit(&self, _: &str, _: Value) -> Result<(), String> {
        Ok(())
    }
}
#[rustfmt::skip]
#[path = "fixtures/beta/common.rs"]
pub mod common;
mod native_core {
    pub use crate::common;
}
#[allow(dead_code)]
#[rustfmt::skip]
#[path = "fixtures/beta/v2_sdk.rs"]
mod v2_sdk;
#[path = "../src/tezbar_adapter.rs"]
mod adapter;

type Reply = Result<Value, String>;
type Calls = Arc<Mutex<Vec<(String, Vec<Value>)>>>;
fn context(replies: Vec<Reply>) -> (common::CoreContext, Calls, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let ctx = common::CoreContext::headless(common::CorePaths {
        data_dir: dir.path().join("data"),
        config_dir: dir.path().join("config"),
        resource_dir: dir.path().join("resources"),
        temp_dir: dir.path().join("tmp"),
        home_dir: dir.path().into(),
    });
    let calls: Calls = Arc::default();
    let history = calls.clone();
    let replies = Arc::new(Mutex::new(std::collections::VecDeque::from(replies)));
    ctx.set_router(Arc::new(move |channel, args| {
        history.lock().unwrap().push((channel, args));
        let reply = replies
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected OS call");
        Box::pin(async move { reply })
    }))
    .unwrap();
    (ctx, calls, dir)
}
async fn run(ctx: &common::CoreContext) -> common::CoreResult {
    adapter::handle(
        ctx,
        "markdown-to-jira",
        "extension:run-command",
        &json!({"text":"ignored","arguments":{"text":"ignored"}}),
        &mut json!({}),
    )
    .await
}
#[tokio::test]
async fn source_order_empty_unicode_and_strict_success() {
    for (input, converted) in [
        ("", ""),
        ("**Қазақ 😀**", "*Қазақ 😀*"),
        ("```\n*a*\n```", "{code}\n_a_\n{code}"),
    ] {
        for ack in [
            json!({"ok":true}),
            json!({"ok":true,"error":null,"success":true}),
        ] {
            let (ctx, calls, dir) = context(vec![Ok(json!(input)), Ok(ack.clone()), Ok(ack)]);
            assert_eq!(
                run(&ctx).await.unwrap(),
                json!({"ok":true,"mode":"no-view","message":"Successfully converted Markdown to Jira markup!"})
            );
            assert_eq!(
                *calls.lock().unwrap(),
                vec![
                    ("core:platform:selected-text".into(), vec![]),
                    ("core:platform:paste-text".into(), vec![json!(converted)]),
                    ("clipboard:write".into(), vec![json!(converted)]),
                ]
            );
            assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        }
    }
}
#[tokio::test]
async fn reject_bad_selection_before_effects() {
    for bad in [
        Err("secret selected text".into()),
        Ok(Value::Null),
        Ok(json!({"text":"secret selected text"})),
        Ok(json!(42)),
        Ok(json!(false)),
    ] {
        let (ctx, calls, _) = context(vec![bad]);
        let e = run(&ctx).await.unwrap_err();
        assert!(!e.contains("secret selected text"));
        assert_eq!(calls.lock().unwrap().len(), 1);
    }
}
#[tokio::test]
async fn all_partial_effect_acknowledgments_fail_closed_without_retry() {
    let mut bad = vec![Err("secret selected text".into())];
    for value in [
        json!(null),
        json!(true),
        json!("ok"),
        json!([]),
        json!({}),
        json!({"ok":false}),
        json!({"ok":"true"}),
        json!({"ok":true,"success":false}),
        json!({"ok":true,"success":null}),
        json!({"ok":true,"success":1}),
        json!({"ok":true,"error":"secret selected text"}),
        json!({"ok":true,"error":false}),
        json!({"ok":true,"error":{}}),
    ] {
        bad.push(Ok(value));
    }
    for bad in bad {
        for stage in [1, 2] {
            let mut replies = vec![Ok(json!("secret selected text"))];
            if stage == 2 {
                replies.push(Ok(json!({"ok":true})));
            }
            replies.push(bad.clone());
            let (ctx, calls, _) = context(replies);
            let e = run(&ctx).await.unwrap_err();
            assert!(!e.contains("secret selected text"));
            assert!(e.contains("may"));
            assert!(e.contains("No retry"));
            assert_eq!(calls.lock().unwrap().len(), stage + 1);
        }
    }
}
#[tokio::test]
async fn invalid_command_and_operations_have_no_effects_or_state() {
    let (ctx, calls, _) = context(vec![]);
    assert_eq!(adapter::commands(), &["markdown-to-jira"]);
    for op in [
        "extension:run-command",
        "extension:dispose-session",
        "extension:refresh-session",
    ] {
        assert!(
            adapter::handle(&ctx, "unknown", op, &json!({}), &mut json!({}))
                .await
                .is_err()
        );
    }
    for op in [
        "extension:invoke-action",
        "extension:search-text-changed",
        "extension:load-more",
        "launch",
        "unknown",
    ] {
        assert!(
            adapter::handle(&ctx, "markdown-to-jira", op, &json!({}), &mut json!({}))
                .await
                .is_err()
        );
    }
    for op in ["extension:dispose-session", "extension:refresh-session"] {
        let mut state = json!({"sentinel":true});
        assert_eq!(
            adapter::handle(&ctx, "markdown-to-jira", op, &json!({}), &mut state)
                .await
                .unwrap(),
            json!({"ok":true,"mode":"unchanged"})
        );
        assert_eq!(state, json!({"sentinel":true}));
    }
    assert!(calls.lock().unwrap().is_empty());
}
#[tokio::test]
async fn conversion_limits_precede_paste() {
    for (input, message) in [
        (
            "x".repeat(tezbar_markdown_to_jira::MAX_INPUT_BYTES + 1),
            "limit",
        ),
        (
            "*".repeat(tezbar_markdown_to_jira::MAX_INPUT_BYTES),
            "work limit",
        ),
        ("\n|||\n|---|\n\n".to_string(), "table error"),
    ] {
        let (ctx, calls, _) = context(vec![Ok(json!(input))]);
        assert!(run(&ctx).await.unwrap_err().contains(message));
        assert_eq!(calls.lock().unwrap().len(), 1);
    }
}
