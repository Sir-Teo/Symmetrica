# PR Title (Conventional Commit)

## Summary
- What does this change do and why?

## Changes
- **[type]** Core/Feature/Refactor/Fix/Docs/CI
- **[scope]** Affects crates: `expr_core`, `simplify`, ...

## Checklist
- [ ] Conventional Commit message (e.g., feat(core): add canonical Mul sorting)
- [ ] Tests added/updated
- [ ] `cargo fmt` clean
- [ ] `cargo clippy -D warnings` clean
- [ ] CI green (build, tests, docs, audit, deny)
- [ ] Coverage ≥ 80% for `expr_core` and `simplify` (tarpaulin)
- [ ] Docs updated (if applicable)

## Testing
- How to reproduce/verify:

## Notes
- Determinism and canonicalization verified (pointer equality ⇒ structural equality where relevant)
