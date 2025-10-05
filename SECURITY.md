# Security Policy

## Supported Versions

Symmetrica follows [Semantic Versioning](https://semver.org/). Security updates are provided for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 1.x.x   | :white_check_mark: |
| 0.1.x   | :white_check_mark: (until 1.0.0 stable) |
| < 0.1.0 | :x:                |

## Security Considerations

### Mathematical Correctness

Symmetrica is a symbolic computation engine where **mathematical correctness is a security concern**. Incorrect simplifications, derivatives, or solutions could lead to:

- Incorrect scientific/engineering calculations
- Flawed automated reasoning
- Unsafe system behavior in critical applications

**We treat mathematical bugs as security issues** when they could lead to silent incorrect results.

### Resource Exhaustion

Symbolic computation can be resource-intensive. Symmetrica includes safeguards against:

- **Expression swell:** Canonical forms and guards prevent exponential blowup
- **Infinite loops:** Rewrite systems have iteration caps
- **Memory exhaustion:** DAG sharing and resource limits

However, users should:
- Set appropriate resource limits in production environments
- Validate input expressions before processing
- Use timeouts for untrusted computations

### Input Validation

When parsing expressions from untrusted sources:

- **S-expression parser:** Validated and fuzz-tested
- **JSON parser:** Uses standard `serde_json` with validation
- **LaTeX output only:** No LaTeX parsing (output-only to prevent injection)

The parsers are designed to reject malformed input gracefully without panics.

### WebAssembly Security

When using Symmetrica in WASM:

- Resource caps are enforced (time/steps/memory)
- No file system or network access
- Sandboxed execution environment

### Python Bindings

Python bindings (via PyO3) inherit Rust's memory safety but:

- GIL handling is standard PyO3 practice
- No unsafe Python code generation
- Type conversions are validated

## Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues.**

### How to Report

Send security reports to: **security@symmetrica.dev** (or create a private security advisory on GitHub)

Include:

1. **Description:** Clear explanation of the vulnerability
2. **Impact:** Potential security impact and affected versions
3. **Reproduction:** Steps to reproduce the issue
4. **Suggested Fix:** If you have one (optional)

### What to Expect

- **Acknowledgment:** Within 48 hours
- **Initial Assessment:** Within 7 days
- **Status Updates:** Every 7 days until resolved
- **Fix Timeline:** Critical issues within 30 days, others within 90 days

### Disclosure Policy

We follow **coordinated disclosure**:

1. **Private Fix:** We develop and test a fix privately
2. **Advance Notice:** Security advisory published 7 days before release
3. **Public Release:** Fix released with CVE (if applicable)
4. **Credit:** Reporter credited (unless anonymity requested)

### Security Advisories

Published at: https://github.com/Sir-Teo/Symmetrica/security/advisories

## Security Best Practices for Users

### Production Deployments

1. **Resource Limits:**
   ```rust
   // Set appropriate limits for your use case
   let store = Store::new();
   // Validate expression size before processing
   ```

2. **Input Validation:**
   ```rust
   // Always validate untrusted input
   match from_sexpr(&mut store, untrusted_input) {
       Ok(expr) => { /* safe to use */ },
       Err(e) => { /* reject malformed input */ }
   }
   ```

3. **Timeouts:**
   ```rust
   // Use timeouts for potentially expensive operations
   use std::time::Duration;
   // Implement timeout wrapper for long-running computations
   ```

### Sandboxing

For untrusted computations:

- Use WASM build with resource caps
- Run in isolated containers/VMs
- Implement operation timeouts
- Monitor resource usage

### Dependency Security

Symmetrica minimizes dependencies:

- Core functionality: **zero dependencies** (stdlib only)
- Optional features: vetted dependencies only
- Regular `cargo audit` in CI/CD
- `cargo deny` for license/security compliance

## Security Testing

Symmetrica includes:

- **Fuzz Testing:** 4 fuzz targets (parser, simplifier, diff, expr_ops)
- **Property Testing:** Algebraic law verification
- **Differential Testing:** Validation against reference implementations
- **CI Security Checks:** `cargo audit` and `cargo deny` on every commit

## Known Security Considerations

### Not Cryptographically Secure

Symmetrica is **not designed for cryptographic applications**:

- Hash functions are for structural equality, not cryptographic security
- Random number generation (if any) is not cryptographically secure
- No timing-attack resistance guarantees

### Floating-Point Evaluation

The `evalf` module uses floating-point arithmetic:

- Subject to standard floating-point errors
- Not suitable for security-critical numerical computations
- Use exact rational arithmetic when precision matters

### Assumptions System

The assumptions system affects simplification behavior:

- Incorrect assumptions can lead to incorrect results
- Always validate assumption contexts
- Default assumptions are conservative (complex domain)

## Compliance

### License

Dual licensed under MIT OR Apache-2.0 (user's choice).

### Export Control

Symmetrica is open-source mathematical software and is generally not subject to export controls. However, users are responsible for compliance with their local laws.

## Contact

- **Security Issues:** security@symmetrica.dev (or GitHub Security Advisories)
- **General Issues:** https://github.com/Sir-Teo/Symmetrica/issues
- **Discussions:** https://github.com/Sir-Teo/Symmetrica/discussions

## Acknowledgments

We thank the security research community for responsible disclosure and appreciate all reports that help improve Symmetrica's security.

---

**Last Updated:** 2025-10-05  
**Policy Version:** 1.0
