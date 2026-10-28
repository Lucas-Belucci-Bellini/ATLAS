# ATLAS HTTP contract v1

Status: implementation in progress. This is the proposed shared wire contract for ATLAS, TITAN and AETHER. The current client implements browser Fetch, timeout handling and defensive parsing; browser-level verification remains pending.

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

## Versioned error envelope

The family v1 error shape is:

```json
{
  "error": {
    "code": "not_found",
    "message": "recurso não encontrado"
  },
  "meta": {
    "api_version": "v1"
  }
}
```

The client adds `error.code` and `error.message` to its diagnostic only when the response contains this v1 envelope. It always preserves the HTTP status as the primary failure signal and includes `x-request-id` when the server exposes it. Unknown/legacy error bodies are not interpreted as the family contract.

## First reference endpoint

`GET /api/v1/hello`

Expected status: `200 OK`. The reference TITAN endpoint returns a message, environment and API version. It is a smoke-test endpoint, not an authentication or readiness mechanism.

## Client behavior

The browser HTTP client in this branch:
- rejects network failures, non-2xx statuses, invalid success JSON and request timeout expiration;
- supports a configurable absolute HTTP(S) base URL and a per-request timeout;
- parses the shared success envelope on demand and checks required route-specific string fields;
- reads the versioned error code/message when the body opts into contract v1;
- extracts `x-request-id` when available;
- does not send cookies or authentication credentials automatically.

Cross-origin servers must explicitly configure CORS for the frontend origin and expose `x-request-id` if browser code needs to read it. CORS alone is not authentication or authorization. The selected auth model must document credentials, CSRF and origin assumptions.

## Compatibility and security

ATLAS must not depend on TITAN internals or import backend Rust types as a wire format. Use HTTPS in production. Additive fields are preferred; breaking changes require an explicit versioning decision. Never log credentials or sensitive payloads.

## Implementation status

- TITAN `GET /api/v1/hello`: reference success endpoint.
- TITAN route-level 404/405 and request timeouts under `/api/v1`: use the versioned error envelope; unversioned routes retain legacy error shapes.
- ATLAS `HttpClient`, `HttpResponse` and `ApiEnvelope`: implemented in this branch; error diagnostics recognize the v1 error envelope.
- CI native and WASM checks pass for the current implementation branch. The actual browser interaction has not been automated or manually verified yet.
- Cross-repository generated types and automated browser conformance tests: not implemented.
