# Validation evidence

Executed on 2026-10-24. Current toolchain: Rust/Cargo 1.98.0 on aarch64-apple-darwin; MSRV toolchain: Rust 1.85.0. Development oracle: Node 26.7.0, pnpm 11.23.0, TypeScript 5.9.3. This records automated results, with desktop QA kept separate.

| Check | Result |
| --- | --- |
| Actual own TypeScript command plus locked jira2md 3.0.0 / marked 4.3.0 | 17,863 deterministic corpus cases; 644 original library exceptions, all preserved as conversion errors |
| Source command selected-text → conversion → paste → explicit copy → HUD | Executed for every corpus case; empty string included; all four async failure stages executed and caught to failure toast |
| Rust converter against generated library outputs | All corpus cases pass; malformed, mixed/nested markup, Unicode, JS whitespace, dollar replacement tokens, CRLF/CR/LF/U+2028/U+2029 included |
| Adversarial bounds | Input cap, backtracking rejection, table failure and output bound checked; conversion failures precede OS paste |
| Real beta CoreContext + v2_sdk + unchanged production adapter | 10 integration/fixture tests pass; only OS routing/Tauri event boundary stubbed |
| Strict acknowledgments and partial effects | Success objects, malformed selections, transport errors, non-object/false/missing/incorrect acknowledgments, `ok:true` with non-null error, and false/null/non-boolean success tested at paste and copy; stop, redact, no retry |
| Routing/lifecycle | Exact command validation before any effect; launch, harmless refresh/dispose, rejected actions/search/load-more/unknown operations; payload ignored and state unchanged |
| Rust 1.98 and 1.85 tests | Pass on both toolchains, 12 tests including the full corpus |
| Rust 1.98 and 1.85 formatting | `cargo fmt --check` passes on both; beta fixture bytes retained |
| Rust 1.98 and 1.85 lint | `cargo clippy --locked --all-targets -- -D warnings` passes on both |
| Windows GNU | `cargo check` and strict all-target Clippy pass for `x86_64-pc-windows-gnu`; no Windows binaries executed |
| Cargo lock/MSRV | 50 locked third-party packages; all declared MSRVs <= 1.85. Test-only tempfile disables unused getrandom support |
| Full pinned beta draft-2020-12 schema | Validated using jsonschema 4.23.0, including root additional-properties restriction |
| Source/provenance | SHA-256 retained source evidence; npm tarballs' own lockfile SHA-512 integrity; all extracted npm file bytes; oracle executable bytes and corpus digest verified |
| Licenses | Own/upstream MIT, jira2md Apache-2.0, marked MIT + BSD-3-Clause, beta Apache-2.0, actual notices for every locked Rust dependency retained |
| Icon | Own and pinned Raycast files byte-equal, 13,159 bytes, SHA-256 d44cd5561594f166f1414a1adada6c1da8004d9c12dcd2d2ae39b7299a27cd9c |
| Disposable V1 regression | `pnpm test` and `pnpm typecheck` pass after importing original npm lock; original root lockfile remains unchanged |
| Cargo package | Build verification and extracted-package tests/schema/provenance pass; tests, licenses, full npm provenance and oracle fixtures included; target/.work excluded |
| Git scope | Branch feat/native-v2-rust; additions under v2 only; root README discovery link only; every other V1 tracked byte matches the pinned baseline; no staging/commit/push/PR |

The source oracle is development-only and uses synthetic inputs, never the real selection or clipboard. Rust tests consume the retained JSONL fixtures without executing JavaScript. Byte-exact beta fixtures contain their own upstream storage tests; the production Markdown to Jira adapter itself performs no persistence.

The SDK exposes no public HUD/toast route for this use. The port returns native no-view success and sanitized native errors; the original notification UI is a documented gap. No full Tezbar application build or live user-profile installation was performed. Real macOS and Windows selection permissions, clipboard contents, focus restoration and paste QA are **not-run**, as recorded in the manifest.
