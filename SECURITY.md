# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 1.0.x   | :white_check_mark: |
| < 1.0   | :x:                |

## Reporting a Vulnerability

We take the security of Symmetrica seriously. If you discover a security vulnerability, please follow these steps:

1. **Do not** disclose the vulnerability publicly until it has been addressed
2. Report the vulnerability via GitHub Security Advisories at: https://github.com/Sir-Teo/Symmetrica/security/advisories
3. Include detailed information about the vulnerability:
   - Description of the issue
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)

You can expect:
- An initial response within 48 hours
- Regular updates on the progress of addressing the vulnerability
- Credit in the release notes (if desired) when the fix is published

## Security Best Practices

When using Symmetrica:

1. **Input Validation**: Always validate user input before passing it to symbolic computation functions
2. **Resource Limits**: Be aware that complex symbolic computations can consume significant memory and CPU time
3. **Dependencies**: Keep all dependencies up to date to receive security patches
4. **Sandboxing**: Consider running untrusted symbolic computations in a sandboxed environment

## Disclosure Policy

When we receive a security bug report, we will:

1. Confirm the problem and determine affected versions
2. Audit code to find similar problems
3. Prepare fixes for all supported versions
4. Release new versions as soon as possible
5. Publish a security advisory on GitHub

## Known Security Considerations

- **Computational Complexity**: Some symbolic operations may have exponential time or space complexity. Use with caution on untrusted input
- **Stack Overflow**: Deeply nested expressions may cause stack overflow in recursive operations
- **Memory Exhaustion**: Large matrix operations or polynomial computations can consume significant memory

## Contact

For security-related questions or concerns, please use GitHub Security Advisories:
https://github.com/Sir-Teo/Symmetrica/security/advisories
