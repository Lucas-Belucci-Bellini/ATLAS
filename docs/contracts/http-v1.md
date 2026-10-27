# ATLAS HTTP contract v1

Status: implementation in progress. This is the proposed shared wire contract for ATLAS, TITAN and AETHER. The initial client is implemented in the coordinated branch; browser verification is pending.

## Role

ATLAS is the browser/WASM frontend. It should consume versioned HTTP/JSON endpoints through browser-compatible APIs and remain independent of server implementation details. A compatible server need not be TITAN.

## Successful response envelope

Versioned endpoints should return:

```json
{
  "data": {
    "message": "Olá do TITAN!",
    "environment": "development"
  },
  "meta": {
    "api_version": "v1"
  }
}
```

- Route-specific fields belong inside `data`.
- `meta.api_version` identifies the contract version.
- JSON field names use snake_case.
- Clients should tolerate unknown additive fields.
- Responses should use `application/json`.

## First reference endpoint

`GET /api/v1/hello`

Expected status: `200 OK`. The reference TITAN endpoint returns a message, environment and API version. It is a smoke-test endpoint, not an authentication or readiness mechanism.

## Client behavior requirements

The browser HTTP client in this branch currently:
- rejects on network failure, timeout, non-2xx status and JSON parsing failure; the API envelope parser returns an explicit error for missing or incorrectly typed fields;
- never assume that every server returns the same error body until error compatibility is standardized;
- support a configurable base URL and avoid hard-coded production hosts;
- avoid logging credentials or sensitive payloads;
- exposes `x-request-id` in successful `HttpResponse` metadata and includes it in HTTP status errors when the header is available;
- document CORS, credential and CSRF assumptions for the selected authentication model.

## Compatibility and security

ATLAS must not depend on TITAN internals or import backend Rust types as a wire format. Use HTTPS in production. CORS alone does not provide authentication or authorization. Additive fields are preferred; breaking changes require an explicit versioning decision.

## Implementation status

- Contract documentation: proposed in all three repositories.
- TITAN `GET /api/v1/hello`: added in the coordinated branch and covered by a dedicated integration test (execution pending).
- ATLAS `HttpClient`, `HttpResponse`, and shared-envelope parser: added in the coordinated branch, including per-request timeout and request-ID extraction.
- Browser/WASM build and browser-level integration test: pending; native tests do not execute the WASM-only client.
- AETHER adapter/conformance tests: not implemented by this document.
- Tests/builds: not run by the authoring action; verify through CI or local execution before claiming success.
