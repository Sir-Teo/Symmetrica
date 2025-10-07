# Coverage Improvements

This document tracks areas of the codebase that would benefit from additional tests and suggested strategies to raise coverage.

## Targets
- calculus/integrate: broaden rational/log edge cases and improper integrals.
- simplify/log_simplify: branch-cut guarded transformations with assumptions.
- special functions: series and numeric approximations for Gamma/Ei.
- grobner: S-polynomial and reduction once implemented.

## Strategies
- Property-based tests with proptest for identities and differential checks.
- Differential testing against reference CAS for selected classes.
- Code coverage with tarpaulin: focus on low-hit lines in reports.
