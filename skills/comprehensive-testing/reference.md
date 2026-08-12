# Comprehensive Testing Reference

Use this file only when `SKILL.md` says deeper guidance is needed. The main skill owns the workflow and gates; this file expands the testing methods.

## 1. Mental Model

Regression hardening asks a different question from implementation-time testing:

```text
Implementation-time testing:
Can we build this behavior correctly now?

Regression hardening:
If a future change breaks confirmed behavior, will the automated test suite catch it?
```

The goal is not maximum test count. The goal is durable protection of behavior that matters.

A useful regression suite protects:

```text
confirmed behavior
+ meaningful boundaries
+ important state/path rules
+ historical failures
+ high-risk invariants
+ critical user-visible flows
```

while avoiding tests that merely freeze implementation details.

---

## 2. CTFL-Inspired Regression Audit

CTFL-style techniques help systematically search for missing test dimensions. In regression hardening, use them as an **audit lens**, not as a requirement to generate every theoretical test.

### 2.1 Start from behavior contracts

Prefer:

```text
requirements / acceptance criteria
↓
approved spec or design
↓
confirmed bug behavior
↓
existing tests / historical evidence
↓
production code
```

For each behavior, express:

```text
input / event
+
precondition / state
↓
expected result
+
forbidden result
```

If the expected behavior cannot be stated clearly, do not guess it into a test.

### 2.2 Equivalence classes

Group inputs that should produce the same system behavior.

Example for a path-sensitive feature:

```text
valid path
invalid path
missing path
regular file
directory
symlink
```

Regression value comes from covering **behaviorally distinct classes**, not many samples from the same class.

Ask:

- Which categories are already protected?
- Which categories have meaningfully different behavior?
- Which unprotected class could realistically regress?

### 2.3 Boundary values

Bugs cluster where rules change.

For an allowed range `1..100`, the meaningful audit points are often:

```text
0
1
2
99
100
101
```

Also consider semantic boundaries:

```text
empty
zero
one item
maximum allowed size
just above limit
first / last state
before / during / after transition
```

Add a regression test only when the boundary has distinct behavior or meaningful failure risk.

### 2.4 Decision tables / condition combinations

When behavior depends on multiple conditions, list the important combinations instead of relying on intuition.

Example:

```text
path valid?
target exists?
user approved?
operation succeeds?
→ expected outcome
```

Do not blindly enumerate the full Cartesian product. Prioritize:

- Every important outcome.
- Conditions whose true/false value changes behavior.
- Conflicting combinations.
- High-risk combinations.
- Combinations implicated by previous bugs.

### 2.5 State transitions

For stateful behavior, inspect:

- Important states.
- Allowed transitions.
- Forbidden transitions.
- Failure transitions.
- Interruptions.
- Recovery.
- Re-entry / repeated operations.

When this becomes difficult to reason about as a list, switch to CT-MBT.

### 2.6 Failure and recovery

Actively inspect realistic failures:

```text
invalid input
dependency failure
timeout
cancel
duplicate action
partial success
missing resource
permission failure
persistence failure
platform difference
recovery after failure
```

Prioritize by:

```text
impact × likelihood × regression susceptibility
```

This is a prioritization heuristic, not a numeric scoring requirement.

---

## 3. CT-MBT: Model-Based Regression Audit

CT-MBT is useful when a flat scenario list no longer gives confidence that important paths are covered.

### 3.1 Trigger conditions

Strong signals:

- Multiple states.
- Multi-stage workflow.
- Same event behaves differently by state.
- Retry.
- Approval.
- Cancellation.
- Timeout.
- Recovery.
- Repeated tool / operation calls.
- Asynchronous events.
- Concurrency or ordering.
- Many paths that are hard to enumerate reliably.

Simple input → processing → output code usually does not need a model.

### 3.2 Minimum model

Do not build elaborate UML unless the project already uses it.

Capture:

```text
state
transition
trigger / condition
expected result
invariant
```

Example:

```text
Idle
 ↓ submit
Streaming
 ↓ tool_call
ToolExecuting
 ├─ success → Streaming
 ├─ failure → Streaming
 └─ cancel  → Cancelled
                  ↓ reset
                 Idle
```

For a critical transition:

```text
From:
ToolExecuting

Trigger:
cancel

To:
Cancelled

Expected:
active operation stops / cancellation state is committed

Invariant:
stale result must not be accepted after cancellation
```

### 3.3 Audit the model against tests

Check:

#### State coverage
Can tests reach every important state?

#### Transition coverage
Does every important legal transition have protection?

#### Forbidden transitions
Are important invalid transitions rejected?

#### Condition branches
When a transition depends on conditions, are the behavior-changing branches protected?

#### Interrupt / recovery
At each meaningful interruption point:

- Does the system enter the right state?
- Are stale events ignored?
- Can it recover or continue correctly?

#### Critical paths
Typical stateful-system paths may include:

```text
normal completion
success path
failure path
repeat operation
cancel during operation
approval rejection
dependency/provider failure
maximum-iteration / retry limit
recovery after failure
```

The goal is not every possible path. The goal is confidence in states, transitions, invariants, and high-risk end-to-end routes.

---

## 4. CT-GenAI Guardrails

When an AI agent performs regression hardening, its main risk is reproducing its own misunderstanding.

### 4.1 Do not let implementation become the specification

Bad loop:

```text
wrong implementation
↓
AI reads implementation
↓
AI writes tests matching it
↓
tests pass
```

Use implementation to understand *how* behavior is produced, not *what behavior is correct*.

### 4.2 Preserve traceability

Every important added test should answer:

```text
Why does this test exist?
```

It should trace to at least one of:

- Behavior contract.
- Boundary.
- Equivalence class.
- Condition combination.
- State transition.
- Invariant.
- Historical bug.
- High-risk user/system path.

### 4.3 Expose uncertainty

Separate:

```text
confirmed facts
assumptions
unknowns
tests blocked by unknowns
```

Do not silently turn an assumption into a permanent regression contract.

### 4.4 Do not let the same AI output self-certify

Passing generated tests are not sufficient proof.

Use independent evidence where valuable:

- Known pre-fix behavior.
- Reversible targeted mutation.
- Property testing.
- E2E.
- Fuzzing.
- Mutation testing.

---

## 5. Choosing the Test Level

Use the lowest level that still preserves the real failure mode.

### Unit

Best when one component or rule fully expresses the behavior.

Strong for:

- Pure logic.
- Parsing.
- Configuration rules.
- Data conversion.
- Local state transitions.

Advantages:

```text
fast
stable
easy to diagnose
```

Do not force unit tests when the regression lives in interaction between components.

### Integration

Use when the risk is in collaboration:

```text
component A
+
component B
+
data/state flow
```

Typical failures:

- Wrong call order.
- State not synchronized.
- Data shape mismatch.
- Error not propagated.
- Cleanup not executed.
- Real dependency semantics differ from mocks.

### E2E

Use when the behavior only matters at the real user/system boundary.

Examples:

```text
user input
→ interface
→ application state
→ external interaction
→ final visible result
```

E2E should protect critical flows, not duplicate every lower-level case.

---

## 6. Property-Based Testing

Property testing is valuable when the important contract is an invariant across many inputs.

Instead of:

```text
input A → expected
input B → expected
```

define:

```text
for all valid x:
property(x) holds
```

Example:

```text
decode(encode(x)) == x
```

Good candidates:

- Parsers.
- Path normalization.
- Configuration transforms.
- Serialization.
- Tool parameters.
- Data structures.
- State-machine invariants.

Prefer property testing when hand-picked examples cannot give confidence across the input space.

Do not use it when the behavior is best expressed by a few concrete business examples.

---

## 7. Fuzz Testing

Fuzz testing attacks the system with many generated or malformed inputs to discover:

```text
crash
panic
out-of-bounds behavior
invariant violation
unexpected parser/protocol failure
```

Strong candidates:

- Parsers.
- File formats.
- Protocols.
- Network/untrusted input.
- Tool arguments.
- Complex binary/text input surfaces.

Use fuzzing for robustness exploration, not as a replacement for precise regression assertions.

Property testing asks:

```text
Does a known property hold across many inputs?
```

Fuzzing asks:

```text
Can an unexpected input break the program?
```

---

## 8. Mutation Testing

Mutation testing evaluates the **tests**, not just the production code.

It deliberately introduces plausible mistakes such as:

```text
>  → >=
true → false
+ → -
remove a condition
remove a call
change a return value
```

Then re-runs tests.

```text
tests fail
→ mutant killed
→ tests detected the mistake

tests still pass
→ mutant survived
→ possible regression gap
```

Coverage asks:

```text
Was this code executed?
```

Mutation asks:

```text
If this code were wrong, would the tests notice?
```

Mutation testing is especially useful for:

- Critical branching logic.
- Validation / security boundaries.
- State-transition rules.
- Historical regression hot spots.
- Code with high coverage but uncertain assertion strength.

Do not require 100% mutation score. Investigate meaningful survivors and ignore equivalent or irrelevant mutations with documented reasoning.

---

## 9. High-Value Regression Tests

A regression test is high value when it protects a distinct behavior that could realistically break.

Strong reasons to add:

- Explicit contract.
- Behavior-changing boundary.
- Important condition interaction.
- Critical transition.
- Invariant.
- Historical bug.
- High-impact failure/recovery path.
- User-critical flow.

Weak reasons:

- "Coverage is low."
- "We need more tests."
- "Every function should have a test."
- "This private helper is untested."
- "The framework can do X."
- "Another test already covers the same observable behavior."

Prefer a small suite where every test has a clear reason to exist.

---

## 10. Proving Regression-Test Sensitivity

A passing regression test may still be incapable of catching the regression.

For critical new protection, demonstrate sensitivity.

Preferred evidence:

### Known bad behavior

Run the new test against:

- The pre-fix revision.
- A deterministic reproduction of the original bug.

Expected:

```text
bad behavior → test fails for the intended reason
correct behavior → test passes
```

### Reversible targeted mutation

Temporarily change only the protected behavior.

Example:

```text
expected boundary comparison:
x > limit

temporary mutation:
x >= limit
```

The regression test should fail.

Restore immediately and verify the working tree.

### Mutation tool

For broader critical logic, use a mutation-testing tool and inspect meaningful survivors.

The objective is evidence that the suite can detect realistic mistakes, not mutation score maximization.

---

## 11. Reporting

A useful report explains behavior protection, not just counts.

Weak:

```text
127 tests passed
```

Better:

```text
Behavior audited:
- cancellation
- stale-result suppression
- retry limit

Gaps found:
- cancellation during tool execution lacked regression protection
- retry boundary lacked coverage

Tests added:
- integration regression for stale result after cancel
- boundary regression at max retries

Advanced checks:
- Mutation used on cancellation guard; target mutant killed
- E2E not added: integration test reaches the real failure boundary

Remaining risk:
- OS-specific process termination timing not automated
```

Always distinguish:

```text
proved
not proved
intentionally not tested
unknown
```

That distinction is more useful than raw test count.
