# Security Policy

## Reporting a Vulnerability

We take the security of Cradle Backend seriously. If you discover a security vulnerability, please follow these steps:

1.  **Do NOT open a public issue.** Security vulnerabilities should be handled discreetly to protect our users.
2.  **Email us directly** at `security@cradlemarkets.com`.
3.  Include a detailed description of the vulnerability, steps to reproduce, and any potential impact.
4.  Our security team will acknowledge your report within 48 hours and provide an estimated timeline for a fix.

## Security Controls

As outlined in our [Security Model](SECURITY_MODEL.md):

*   **Private Network Only**: This service is not intended to be exposed to the public internet. Ensure it is deployed within a private subnet or behind a strict VPN/Bastion.
*   **Authentication**: Ensure API keys are rotated regularly and `validate_auth` middleware is active on all non-health endpoints.
*   **Dependencies**: We regularly audit `Cargo.toml` dependencies. Please refrain from adding unverified crates.

## Responsible Disclosure

We ask that you give us reasonable time to fix the issue before discussing it publicly. We will notify you once the fix has been deployed.
