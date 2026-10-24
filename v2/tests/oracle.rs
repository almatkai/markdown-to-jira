use serde_json::Value;
#[test]
fn actual_pinned_library_corpus() {
    let mut count = 0;
    for line in include_str!("fixtures/oracle.jsonl").lines() {
        let case: Value = serde_json::from_str(line).unwrap();
        let got = tezbar_markdown_to_jira::to_jira(case["input"].as_str().unwrap());
        if case.get("error").is_some() {
            assert!(got.is_err(), "case {count}");
        } else {
            assert_eq!(
                got.unwrap(),
                case["output"].as_str().unwrap(),
                "case {count}"
            );
        }
        count += 1;
    }
    assert!(count >= 10_000);
}
#[test]
fn bounded_adversarial_work() {
    use tezbar_markdown_to_jira::{to_jira, ConvertError, MAX_INPUT_BYTES};
    assert_eq!(
        to_jira(&"x".repeat(MAX_INPUT_BYTES + 1)),
        Err(ConvertError::InputLimit)
    );
    assert_eq!(
        to_jira(&"*".repeat(MAX_INPUT_BYTES)),
        Err(ConvertError::WorkLimit)
    );
    for sample in [
        "*".repeat(MAX_INPUT_BYTES),
        "_".repeat(MAX_INPUT_BYTES),
        "|".repeat(MAX_INPUT_BYTES),
        format!("\n{}\n|---|\n\n", "|".repeat(8000)),
        format!("`{}", "a".repeat(16000)),
    ] {
        let result = to_jira(&sample);
        assert!(result.as_ref().map_or_else(
            |_| true,
            |s| s.len() <= tezbar_markdown_to_jira::MAX_OUTPUT_BYTES
        ));
    }
}
