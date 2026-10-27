# Security Policy

## Supported versions

Until a stable release policy is published, security fixes should target the latest development version on the default branch. This is a project policy proposal and does not imply a production-support commitment.

## Reporting a vulnerability

Please do not publish exploit details in a public issue. Use GitHub's private vulnerability reporting or Security Advisories feature for this repository if available. If that feature is unavailable, contact the repository maintainers privately through their GitHub profile and include a minimal reproduction, affected revision, impact, and suggested mitigation where possible.

## Security review priorities

- Escape or safely encode untrusted text and attribute values; do not interpret arbitrary user input as markup.
- Review dynamic attributes, event handlers, and DOM insertion/removal behavior.
- Ensure unmount disposes listeners and reactive scopes to avoid stale callbacks and leaks.
- Treat HTML/DOM/browser APIs as a trust boundary and document unsupported unsafe patterns.
- Run the RustSec advisory audit against Cargo.lock in CI.
- Do not describe the framework as production-ready until browser behavior and security-sensitive rendering are tested.
