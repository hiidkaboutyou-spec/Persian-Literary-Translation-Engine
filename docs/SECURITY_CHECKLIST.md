# Security Checklist

Before release:

- [ ] No secrets committed
- [ ] cargo audit reviewed
- [ ] Dependencies reviewed
- [ ] Imported files treated as untrusted data
- [ ] Database migrations tested
- [ ] Error reporting removes private content
- [ ] Logs do not contain manuscripts
- [ ] Tests cover malformed input

Rust principles:

- Prefer safe Rust
- Avoid unnecessary unsafe blocks
- Use explicit error handling
- Validate external input
