# Compiled host integration

Target API: Tezbar beta `fd300e276356587726566026f072563eb9c34cb1`. The retained [format documentation](tests/fixtures/beta/NATIVE_V2_FORMAT.md), [schema](tests/fixtures/beta/extension-v2.schema.json), `common.rs`, `v2_sdk.rs`, `v2_runtime.rs`, `native.rs` and `platform.rs` fixtures are exact bytes read from that commit. This package does not modify the host or any user profile.

1. Package this directory with `cargo package --allow-dirty --locked`. Extract the resulting `.crate` archive into the host's stable `src-tauri/native-extensions/markdown-to-jira/` directory, stripping its single package-directory prefix. Retain `Cargo.toml`, lockfile, `src`, manifest, tests, oracle fixtures, vendor evidence, scripts and licenses. Do not vendor `target` or `.work`. This works independently of any implementation checkout.
2. Add this path dependency to host `src-tauri/Cargo.toml`:

   ```toml
   tezbar-markdown-to-jira = { path = "native-extensions/markdown-to-jira" }
   ```

   The host already supplies `serde_json`, the real `CoreContext` and real `v2_sdk`. Do not replace them with test fixtures.
3. In `src-tauri/src/native_core/extensions/mod.rs`, include the production adapter as a sibling of `v2_sdk`:

   ```rust
   #[path = "../../../native-extensions/markdown-to-jira/src/tezbar_adapter.rs"]
   mod v2_markdown_to_jira;
   ```

4. Extend all three compiled match tables in `src-tauri/src/native_core/extensions/v2_runtime.rs`:

   - Add `"markdown-to-jira"` to the allowlist in `slug()`. Existing prefix handling provides `com.tezbar.native.markdown-to-jira` and `raycast.markdown-to-jira` aliases.
   - Add `"markdown-to-jira" => super::v2_markdown_to_jira::commands(),` to `commands()`.
   - Add the handler arm below to `handle()`.

   ```rust
   "markdown-to-jira" => {
       super::v2_markdown_to_jira::handle(ctx, command, operation, payload, state).await
   }
   ```

5. Append the object in `manifest/extension-v2.json` to the bundled array in `src-tauri/src/native_core/extensions/v2_catalog.json`. It is schema 2; root `author` is intentionally absent because the pinned schema disallows it. Credits are in `description`. Update the catalog test's exact extension count for the new entry. Coverage must continue to derive from compiled dispatch and platform declarations.
6. Copy the exact bundled icon from `manifest/extensions/icons/markdown-to-jira.png` to **`src/renderer/public/extensions/icons/markdown-to-jira.png`** in the host. The manifest URL is `/extensions/icons/markdown-to-jira.png`. Its 13,159 bytes have SHA-256 `d44cd5561594f166f1414a1adada6c1da8004d9c12dcd2d2ae39b7299a27cd9c`.
7. Rebuild the host binary and frontend resources. Use the host's normal canonical Native V2 installation flow after compiled coverage becomes available. No external Rust loader or executable path is involved; copying the JSON alone does not install executable behavior. Existing V1 stays available, and no legacy state needs conversion.

The adapter validates the command before handling lifecycle operations. `extension:run-command` performs conversion; `extension:dispose-session` and `extension:refresh-session` return SDK `unchanged` without effects. Action, search, load-more and all unknown operations return errors. Session payload and state are unused. The normal host installation gate, identity resolution and session wrapper remain host responsibilities.

The only runtime requests are:

| Stage | Real beta route | Arguments | Required response |
| --- | --- | --- | --- |
| Selection | `core:platform:selected-text` | `[]` | String, including empty |
| Paste | `core:platform:paste-text` | `[converted]` | Strict acknowledgment |
| Final copy | SDK `copy` → `clipboard:write` | `[converted]` | Strict acknowledgment |

Strict acknowledgment means object `ok: true`, no non-null `error`, and absent/true `success`. Do not add a HUD/toast channel, clipboard read fallback, extra pre-paste copy, automatic retry or logging of the selected text.

Desktop QA on each supported OS must exercise selected text, empty selection behavior, selection denial, foreground focus, both clipboard writes, successful paste, rejected acknowledgments and failures after a possible paste. Keep verification `not-run` until actual OS evidence exists; a compile or fixture test does not change it.
