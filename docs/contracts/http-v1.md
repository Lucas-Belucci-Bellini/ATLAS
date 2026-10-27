# ATLAS HTTP contract v1

Status: implementation in progress. This is the proposed shared wire contract for ATLAS, TITAN and AETHER, not a claim that ATLAS already has a network client.

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

When the network client is implemented, it must:
- treat network failure, timeout, non-2xx status, invalid JSON and schema mismatch as distinct recoverable errors;
- never assume that every server returns the same error body until error compatibility is standardized;
- support a configurable base URL and avoid hard-coded production hosts;
- avoid logging credentials or sensitive payloads;
- expose `x-request-id` when present for diagnostics;
- document CORS, credential and CSRF assumptions for the selected authentication model.

## Compatibility and security

ATLAS must not depend on TITAN internals or import backend Rust types as a wire format. Use HTTPS in production. CORS alone does not provide authentication or authorization. Additive fields are preferred; breaking changes require an explicit versioning decision.

## Implementation status

- Contract documentation: proposed in all three repositories.
- TITAN `GET /api/v1/hello`: reference implementation being added.
- ATLAS network client and contract tests: not implemented by this document.
- AETHER adapter/conformance tests: not implemented by this document.
- Tests/builds: not run by the authoring action; verify through CI or local execution before claiming success.
