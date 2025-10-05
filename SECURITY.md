# Security Policy

## Supported Versions

We support the latest stable release and the current release candidate for security updates.

- 1.0.0 (stable) — Supported
- 1.0.0-rc.1 (release candidate) — Supported for critical fixes
- Older pre-release versions — Not supported

## Reporting a Vulnerability

If you discover a security issue, please report it responsibly:

- Email: security@symmetrica.dev
- GitHub: Open a private Security Advisory via GitHub Security Advisories

Please include:
- A detailed description of the issue
- Steps to reproduce (if applicable)
- Affected versions and environment
- Possible impact and any known mitigations

We aim to acknowledge reports within 48 hours and provide a timeline for a fix.

## Security Best Practices

- Use the latest stable release
- Enable feature flags only when needed
- Validate and sanitize all untrusted inputs
- Set resource limits to prevent DoS (see `ResourceLimits` in `docs/roadmap.md`)
- Prefer immutable data structures and avoid `unsafe` code

## Disclosure Policy

- We follow a coordinated disclosure process
- We will publish security advisories with CVE details when applicable
- We provide patches or workarounds prior to public disclosure when possible

Thank you for helping keep the Symmetrica community secure.
