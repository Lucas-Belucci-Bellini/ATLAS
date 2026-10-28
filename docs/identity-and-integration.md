# ATLAS family identity and frontend integration contract

Status: **proposed architecture decision for review** (2026-10). This document defines intended product boundaries; it does not claim backend integration or browser tests already exist.

## Official product names

- **AETHER** — modular Rust application/composition framework.
- **ATLAS** — Rust frontend framework for browser interfaces compiled to WebAssembly (WASM).
- **TITAN** — Rust backend HTTP framework for APIs and services.

These are the public product/repository names for the family. The `patina-*` Cargo namespace remains in place during a compatibility transition, while `AtlasView`, `AtlasComponent`, `AtlasElementBuilder`, `AtlasSignal` and `AtlasEffect` are the recommended public type names for new code. Generic names and the earlier `Patina*` aliases remain available for compatibility. Cargo packages/imports and environment-variable names are not changed in this API pass; any package rename needs a separate migration plan and impact inventory.

## ATLAS responsibility

ATLAS owns frontend concerns: declarative views, reactive state, browser DOM rendering, browser events, component composition and (when implemented) routing/build tooling. The current README describes a WASM SPA direction. Do not describe SSR, hydration, hot reload, router or production browser testing as implemented unless code and automated evidence confirm it.

ATLAS should remain independent of server implementation details. It communicates with backend services over documented HTTP APIs, not by importing TITAN internals or sharing Rust types as an implicit network protocol.

## Intended family flow

`ATLAS (browser/WASM) → versioned HTTP/JSON API → TITAN (backend service)`

AETHER is optional. An application may use ATLAS with another compatible HTTP backend, or use TITAN without ATLAS. AETHER may provide a broader application/module orchestration layer but must not become a required runtime dependency for frontend rendering.

## Frontend/backend contract

1. Define endpoints, methods, request/response JSON schemas, status codes and authentication expectations in a versioned API contract.
2. Use stable machine-readable error codes and a documented error envelope. The current TITAN and AETHER README examples differ; compatibility must be mapped and tested rather than assumed.
3. Treat browser input and server responses as untrusted. Avoid unsafe HTML injection; prefer text nodes and explicit, reviewed APIs for any HTML rendering.
4. Document credential transport, CORS requirements and CSRF considerations according to the chosen authentication model. CORS alone is not authorization.
5. Correlation/request IDs should be exposed in a documented response header and included in client-side diagnostic context when available; never log credentials or sensitive payloads.
6. Backend API changes must be reviewed for frontend compatibility. Breaking changes require explicit versioning and migration notes.
7. Keep network client abstractions optional and avoid binding ATLAS to TITAN-specific endpoints or implementation types.

## Compatibility and crate naming

- Product/repository brand: ATLAS.
- Current package namespace: `patina-*` (observed in the workspace manifest).
- Public Rust API: additive `Atlas*` aliases are the recommended names for new code; generic names and `Patina*` aliases remain available.
- A future Cargo package/crate rename must inventory imports, examples, docs, Cargo metadata, feature names and downstream use. No package rename is included in this change.

## Verification gates before claiming integration

- Unit tests for view and reactive semantics.
- Browser/WASM tests for event lifecycle, mounting/unmounting and escaping behavior.
- A contract test against a documented API fixture or a running compatible backend.
- Explicit handling of network errors, timeouts, invalid JSON and non-success status codes.
- Accessibility checks for rendered controls and documented browser support.
- Build/test commands and toolchain versions recorded with actual results.

These are required gates, not claims that the tests currently exist or pass.

## Current evidence and limits

The current README describes `patina-reactive`, `patina-core`, `patina-dom` and `patina`; it also says browser listener tests and several higher-level frontend features are absent. This document does not re-run tests or independently verify those statements.

## Decision record

- Proposed: ATLAS = frontend/WASM framework.
- Proposed: integrate with TITAN via versioned HTTP/JSON contracts only.
- Deferred: server-rendering architecture, a mandatory AETHER dependency, package renaming and any claim of stable cross-framework compatibility.
- Approval: pending maintainer review.
