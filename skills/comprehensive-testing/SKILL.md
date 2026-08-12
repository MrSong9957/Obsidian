---
name: comprehensive-testing
description: Use when a new feature, important module change, or high-risk fix is functionally complete and its planned tests pass, before review, when systematic regression hardening is needed.
---

# Comprehensive Testing

## Overview

Strengthen regression protection for a completed change before review.

**Core principle:** Protect confirmed behavior with the smallest set of high-value tests that catch realistic future regressions.

This skill does not replace implementation-time TDD.

## Entry Gate

Start only when:

- The change is functionally complete.
- Planned / implementation-time tests are green.
- Expected behavior is grounded in requirements, acceptance criteria, an approved spec/design, or confirmed bug behavior.

Otherwise return to implementation/debugging.

## Source of Truth

Priority:

1. Requirements / behavior contracts / acceptance criteria.
2. Approved spec or design.
3. Confirmed bug reproduction / regression contract.
4. Existing tests and historical bug evidence.
5. Production code only as implementation evidence.

Never let the current implementation define correctness by itself. If behavior is ambiguous or sources conflict, surface it rather than encoding a guess. If the audit shows the current implementation contradicts authoritative expected behavior (requirements, acceptance criteria, an approved spec/design, or a confirmed contract), the change is not functionally complete: stop regression hardening and return to implementation/debugging. Do not add permanent regression tests that leave the suite failing — a temporary diagnostic test may be used to demonstrate the mismatch and then removed.

## Workflow

### 1. Inventory
Map confirmed changed behavior to existing tests. Record high-risk paths, historical bugs, and unclear behavior.

### 2. Audit
Check for meaningful gaps in:

- Equivalence classes / input categories.
- Boundaries.
- Important condition combinations.
- States / transitions.
- Failure, interrupt, and recovery paths.
- Relevant timing, external-failure, platform, and historical-bug risks.

This finds candidates; **risk decides what to add**.

### 3. Model if needed
Use CT-MBT when state/path complexity is hard to reason about as a flat list: multi-stage workflows, approval, retry, cancel, timeout, recovery, repeated operations, async events, or concurrency.

Minimum model: **states, allowed/important forbidden transitions, triggers, expected results, invariants, must-cover paths**.

When triggered, **MUST read the CT-MBT section in `reference.md`**.

### 4. Prioritize
Add tests only for distinct contracts, real boundaries/categories, important combinations, critical transitions/invariants, historical bugs, high-risk logic, user-critical paths, or realistic failure/recovery paths.

Reject coverage/count chasing, duplicates, private implementation assertions, trivial framework tests, and mock-only behavior.

### 5. Choose the lowest sufficient level

| Technique | Use when |
|---|---|
| Unit | One component/rule proves the behavior. |
| Integration | Risk is in module contracts or real interaction. |
| Property | An invariant spans a large input space. |
| E2E | The regression matters at the real user/system boundary. |
| Fuzz | Robustness against difficult/untrusted input matters. |
| Mutation | Test effectiveness itself needs evidence. |

If non-obvious, **MUST read the relevant section in `reference.md`**.

### 6. Harden
Add the minimum tests needed. Each important test must state what behavior/boundary/transition/bug/risk it protects.

Prefer real observable behavior; mock only uncontrollable external dependencies. Keep tests independent. Do not refactor unrelated production code.

### 7. Prove sensitivity
Critical new regression tests must fail for the intended reason against known bad behavior, a reversible targeted mutation, or targeted mutation testing; then restore and re-run green.

Leave no temporary mutations behind.

### 8. Verify and report
Run targeted tests while iterating, then project-required affected/full verification. Report protected behavior and remaining risk, not only pass counts.

## Output Contract

Report:

- Behavior audited
- Existing protection
- Gaps found
- Tests added / strengthened
- Property / E2E / Fuzz / Mutation: used or not, with reason
- Rejected candidates and reason
- Remaining risks / unknowns
- Verification: command → result

## Reference Loading Rules

Read the relevant `reference.md` section when:

- CTFL partition/boundary/condition analysis is non-obvious.
- CT-MBT is triggered.
- Advanced-technique selection is non-obvious.
- Minimal high-value coverage is unclear.
- A formal/comprehensive regression audit is requested.

Do not load the whole reference when one section is enough.

## Red Flags

- Planned tests are still failing.
- Expected behavior is inferred only from current code.
- Tests are being added just for coverage/count.
- Simple input/output code is being over-modeled.
- A critical new test passes but its sensitivity was never demonstrated.
