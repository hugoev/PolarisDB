# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

We take security seriously. If you discover a security vulnerability in PolarisDB, please report it responsibly.

### How to Report

1. **Do NOT open a public issue** for security vulnerabilities
2. Email security concerns to: [security@example.com] (replace with your email)
3. Include:
   - Description of the vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if any)

### What to Expect

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Timeline**: Depends on severity, typically 30-90 days

### Disclosure Policy

- We follow responsible disclosure practices
- Security fixes will be released as soon as possible
- Credit will be given to reporters (unless anonymity is preferred)

## Security Best Practices

When using PolarisDB:

1. **File Permissions**: Ensure collection directories have appropriate permissions
2. **Input Validation**: Validate vector dimensions before insertion
3. **Resource Limits**: Monitor memory usage for large datasets
4. **Backup**: Regularly backup collection directories

## Scope

This security policy covers:
- The PolarisDB core library (`polarisdb-core`)
- The main PolarisDB crate (`polarisdb`)
- Official examples and documentation

Third-party integrations and forks are not covered by this policy.
