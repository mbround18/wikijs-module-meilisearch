# Copilot instructions for wikijs-module-meilisearch

These notes teach AI coding agents how to work productively in this repo. Keep edits small, follow existing patterns, and prefer concrete changes over broad refactors.

## Architecture at a glance

- Purpose: Wiki.js search module backed by Meilisearch. Ships as a WASM package plus a tiny runtime image that copies module assets into the Wiki.js container.
- Rust (wasm): `src/lib.rs` exposes `WikiSearchEngine` via wasm-bindgen with async methods: `new(host, key, index, timeout)`, `healthcheck`, `activated`, `suggest`, `query`, `created`, `updated`, `deleted`.
- Rust (binary tool): `src/bin/module_copy.rs` builds `/wiki_meilisearch` with a `copy` subcommand to copy module files from `SOURCE` to `DESTINATION` (defaults: `/modules/meilisearch` → `/wiki/server/modules/meilisearch`). Used in the runtime image.
- JS module wrapper: `engine.js` adapts Wiki.js module lifecycle to the WASM API and maps fields for Wiki.js (e.g., `localeCode` → `locale` for results). Also enforces skipping private/unpublished pages.
- Packaging: artifacts live under `pkg/` (`meilisearch.js`, `meilisearch_bg.wasm`, types). `definition.yml` declares module props shown in the Wiki.js admin UI.
- Delivery: Multi-stage Dockerfile builds WASM + binary, then a distroless runtime copies assets into a Docker volume that Wiki.js mounts.

## Key workflows

- Build local wasm + assets: `make build` (copies `engine.js` and `definition.yml` to `pkg/`, then `wasm-pack build --target nodejs`). Ensure `setup` ran once to install toolchain.
- Dev stack: `make dev` (docker compose up Meilisearch + Wiki.js + module-copy helper). Meilisearch default master key is `demo` (see `compose.yml`).
- Image build/publish: `make compose-build` / `make compose-push` (uses `VERSION` env; default is `sha-<git short>`). Runtime image entrypoint is the copy utility.
- Lint/tests: `make lint` (Rust fmt+clippy, Prettier for js/md/yaml). Rust tests live alongside code (`#[cfg(test)]` in `page.rs`, `query.rs`, `logger.rs`, and binary tests in `module_copy.rs`). Run `cargo test` as needed.

## Conventions and patterns

- Serde naming: Rust uses `#[serde(rename_all = "camelCase")]` for Wiki.js page structs. Keep field names camelCase on the wire.
- Rename events: `RenamedWikiPage` carries `destinationPath`/`destinationHash`; it converts to `WikiPage` via `impl From<RenamedWikiPage>` using destination values. When touching rename flows, update both Rust and JS expectations.
- Suggestions: Built from result content lines in `src/query.rs` using a glob match plus regex cleanup. Deduplicated via `HashSet` and excludes exact-line matches of the query. Preserve this logic unless there’s a clear functional change.
- Logging: `WasmLogger` bridges Rust logs to `WIKI.logger` (Info/Warn/Error). In Node-like contexts outside Wiki.js, you must provide a `global WIKI.logger` shim to avoid extern failures.
- Result shaping: JS `engine.js` maps `localeCode` → `locale` in search results to match Wiki.js expectations.

## Integration points

- Module props (user-configurable): see `definition.yml`
  - `meilisearchHost`, `meilisearchMasterKey`, `indexName`, `timeout`.
- Runtime env overrides (engine.js): `MEILISEARCH_HOST`, `MEILISEARCH_MASTER_KEY`, `MEILISEARCH_INDEX_NAME`, `MEILISEARCH_TIMEOUT` are applied if `process.env` exists.
- Index prep: `WikiSearchEngine.activated()` ensures index exists with primary key `id`, polls readiness, sets filterable attributes: `path`, `hash`, `id`.
- Page lifecycle: `created/updated` add documents, `deleted` removes by `id` filter. `renamed` calls `updated` with the page carrying destination fields. `rebuild` streams from Wiki.js DB (via knex) to reindex published, non-private pages.

## Files you’ll touch most

- Rust: `src/lib.rs`, `src/page.rs`, `src/query.rs`, `src/logger.rs`
- JS: `engine.js` (copied into `pkg/` during build)
- Packaging/ops: `Dockerfile`, `compose.yml`, `Makefile`, `definition.yml`, `pkg/package.json`

## Gotchas

- WASM target and Node: `engine.js` uses `require("./meilisearch.js")`. Prefer `wasm-pack --target nodejs` locally to match this; the Dockerfile currently uses `--target bundler` but still emits `meilisearch.js`. Keep wrapper compatibility in mind if changing targets.
- `WIKI.logger` must exist at runtime for Rust logs; don’t call wasm functions outside Wiki.js without a shim.
- Deleting documents: deletion uses a filter `id = {page.id}`; ensure `id` is numeric in Meilisearch index.

## Practical examples

- Add a new searchable attribute: update `.with_attributes_to_search_on([...])` in `src/lib.rs` and adjust Meilisearch settings if needed.
- Extend suggestion rules: change the regex/glob in `src/query.rs` and update nearby tests.
- Change where assets copy to: adjust `ENV_SOURCE`/`ENV_DESTINATION` defaults in `src/bin/module_copy.rs` and compose env/volumes accordingly.
