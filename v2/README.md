# Markdown to Jira — Rust Native V2

Convert the selected text into Jira wiki markup, paste it into the foreground app, then explicitly copy the same result to the clipboard. This is a text converter; it does not connect to Jira.

The port follows the own extension at `4a6c69069e2bcece239ff42f0755d5b0d054915c` and the actual lockfile's `jira2md` **3.0.0** (`marked` **4.3.0**). Original Raycast author: **theo_brlle**, at `raycast/extensions` commit `6734d85992290066fd31a7f5bee3cca3d3545a1a`. Tezbar V1: **almatkai**. See [provenance](provenance.json), [notices](NOTICE.md) and [validation](VALIDATION.md).

## Behavior

The only input is `core:platform:selected-text` with no arguments; its response must be a string. There is no clipboard read fallback, form, argument input, preference, storage or migration. An empty selected string is converted and passed to both write routes, just as in the source.

Conversion ports `jira2md.to_jira`'s exact ordered regex rules: tables/panels, emphasis, hash and underline headers, ordered and star lists, `<del/ins/sup/sub>` tags, whitespace-delimited strikethrough, code fences, inline code, images, named and angle links, then blockquotes. It is not a Markdown parser or renderer. Malformed syntax and literal library bugs are retained. For example:

- `**bold**` becomes `*bold*`; `***both***` becomes `_*both*_`.
- `~~edge~~` remains unchanged; ` ~~inside~~ ` becomes ` -inside- `.
- Emphasis is converted inside code fences because it runs first.
- Only the library's leading-blank-line table shape matches; aligned `:---:` separators are unsupported. A single-column, single-row table becomes a panel. Empty header matches can throw in the original; Rust returns a safe conversion error before paste.
- JavaScript whitespace (including BOM), ASCII digits, dot exclusions and multiline CR/LF/U+2028/U+2029 assertions are explicitly preserved. String replacements insert captured text literally, including `$&`, `$1`, `$$`, dollar-backtick and dollar-apostrophe.

After complete conversion, the adapter awaits `core:platform:paste-text [result]`. Beta's route writes the clipboard, hides the launcher, restores focus through the platform flow and sends paste. After its acknowledgment, the adapter calls the **actual** SDK `copy`, which requests `clipboard:write [result]` again. Each effect must acknowledge with an object whose `ok` is exactly `true`, `error` is absent or null, and `success` is absent or exactly `true`.

Success returns the source message, `Successfully converted Markdown to Jira markup!`, in the SDK's native no-view response. Beta has no public HUD/failure-toast RPC for this adapter: failures return native errors with the source failure title and safe stage detail. They do not reproduce the original HUD/toast notification UI. OS error details are discarded because they can contain selected text. The adapter does not log user text.

A paste or copy error can follow a partial OS effect. No automatic retry or rollback occurs. Manually relaunching can repeat an unacknowledged paste. A successful acknowledgment confirms the host route completed, not that the target app accepted the text.

## Limits

Input is valid Rust UTF-8, at most **16,384 bytes**. Lone JavaScript UTF-16 surrogates cannot be represented and are outside this API. Intermediate/final output is limited to **131,072 bytes**, with checked appends. There are at most **32,768** capture matches across all stages and table callbacks, and **100,000** fancy-regex backtracking steps per search. Input, output and work limits fail clearly before any OS paste; text is never silently truncated. This adds bounded rejection of unusually large or adversarial input compared with the unbounded library. There are no intentional ordinary-output fixes.

The runtime is entirely Rust and offline apart from the three OS routes above. Vendored npm code is retained for provenance and development oracles, and is never executed by the native adapter. No JS/Bun runtime, subprocess, browser, HTML UI or network client is used by the port.

## Build and integration

Rust MSRV: **1.85**. `Cargo.lock` is retained. The standalone library exports `to_jira(&str) -> Result<String, ConvertError>`. The concrete production bridge is [src/tezbar_adapter.rs](src/tezbar_adapter.rs), compiled unchanged by the beta fixture tests.

```sh
cargo test --locked
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo +1.85.0 test --locked
cargo +1.85.0 clippy --locked --all-targets -- -D warnings
cargo check --locked --all-targets --target x86_64-pc-windows-gnu
cargo clippy --locked --all-targets --target x86_64-pc-windows-gnu -- -D warnings
```

[INTEGRATION.md](INTEGRATION.md) describes stable vendoring at `src-tauri/native-extensions/markdown-to-jira`, compiled registration and the required host rebuild. JSON metadata alone cannot dispatch this code. The public icon destination is `src/renderer/public/extensions/icons/markdown-to-jira.png`.

To regenerate the development oracle, run `sh scripts/prepare-oracle.sh`. It creates a disposable V1 install under `.work/own`, imports the retained npm lock into pnpm, verifies executable dependency bytes against the pinned tarballs, tests/typechecks the original source, then executes the actual TypeScript command and locked library. Rust tests use the included synthetic oracle corpus without Node or pnpm.

For full schema and source checks:

```sh
python3 -m venv .work/schema-venv
.work/schema-venv/bin/pip install jsonschema==4.23.0
.work/schema-venv/bin/python scripts/verify.py
```

Real macOS and Windows selected-text, clipboard, foreground-focus and paste QA remains **not-run**. Cross-compilation and stubbed routing do not establish desktop QA.
