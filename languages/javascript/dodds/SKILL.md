---
name: dodds-testing-practices
description: "Pressure-test JavaScript and TypeScript test suites the Kent C. Dodds way: behavior-first, accessibility-first, and network-boundary aware. Use when writing or triaging React Testing Library, DOM Testing Library, Jest, Vitest, user-event, or MSW tests; fixing act warnings; choosing between getByRole, findBy, waitForElementToBeRemoved, test.each, or MSW; or deciding whether a helper abstraction is earning its keep. Trigger keywords: react-testing-library, testing-library, RTL, user-event, MSW, act warning, waitFor, findByRole, getByRole, fake timers, shallow rendering, test IDs, AHA testing."
tags: testing, react, javascript, typescript, testing-library, msw, integration-testing, accessibility, jest, vitest
---

# Kent C. Dodds Testing Practices

This skill is for tests that should fail only when the product meaningfully regresses. If a behavior-preserving refactor breaks the test, assume the test is wrong until proven otherwise.

## Load Rules

- Before changing network mocks or transport seams, READ `references/msw-patterns.md`.
- Before extracting helpers or flattening nested suites, READ `references/aha-testing.md`.
- Before mass-fixing flaky async, query, or assertion behavior, READ `references/anti-patterns.md`.
- Do NOT load `references/msw-patterns.md` for pure reducers/parsers.
- Do NOT load `references/aha-testing.md` for a single short one-off test fix.

## Before You Touch The Test, Ask Yourself

- What visible steady state proves the behavior finished? If you cannot name it, you are about to silence an `act(...)` warning instead of testing the missing branch.
- Am I querying the accessibility tree or my implementation? If the selector mentions classes, containers, component names, or mocked method calls, you are paying for false confidence.
- Is this really a UI test? Pure computation belongs in `test.each`; protocol drift belongs at the network boundary; DOM tests should not be used to prove call-order trivia.
- If this test is slow, am I trading away confidence consciously? `getByRole` is the right default, but its accessibility checks are expensive in huge trees.

## Decision Tree

### 1. Pick the narrowest meaningful test

- Pure function or input/output matrix: `test.each` or `it.each`; do not render DOM.
- UI flow with local state and network: render the real component and intercept HTTP with MSW.
- Third-party widget or composed child is noisy: full render plus one explicit seam mock. Do not shallow-render the whole tree; that hides broken integration.
- Cross-system auth, payment, or navigation: E2E, but establish auth once in fixtures instead of logging in per spec.

### 2. Choose the query with the least regret

- Start with `getByRole(..., {name})` or `findByRole(..., {name})`.
- If the control is `<input type="password">`, skip role queries; the spec gives it no implicit role, so use `getByLabelText`.
- Roles are matched literally: `checkbox` will not match `switch`. If you are testing fallback roles across older environments, use `queryFallbacks: true`; otherwise query the actual role the user gets.
- When a huge DOM makes `getByRole` the bottleneck, scope with `within` first. Only then consider `hidden: true` or a simpler label/text query, and only if you accept that inaccessible nodes may now be included.

### 3. Choose the async primitive by symptom

- Element should appear: `findBy*`.
- Element must disappear after a transient state: `waitForElementToBeRemoved`; it uses `MutationObserver`, not interval polling, and it only works if the element exists first.
- Arbitrary assertion that eventually becomes true: `waitFor` with one throwing assertion. Defaults matter: immediate first run, `50ms` interval, `1000ms` timeout.
- `act(...)` warning: assume the test stopped before the UI settled. Add the missing visible assertion first. Reach for manual `act` only when you are advancing fake timers or resolving promises outside React's callstack.

### 4. Decide whether to abstract

- One or two short tests: duplicate the setup; the repetition is cheaper than indirection.
- Three similar tests, or one difference buried in 40+ lines of setup: create a file-local `setup()` or `renderThing()` with explicit overrides.
- If the helper needs `beforeEach`, shared `let`s, or nested `describe` state to work, the abstraction is already too clever.

## High-Signal Defaults And Thresholds

- `userEvent.setup()` belongs inside each test or a test-local setup helper, not in shared hooks; it carries device state and cross-test reuse creates ghost failures.
- With fake timers, pair `userEvent.setup({advanceTimers: jest.advanceTimersByTime})` or the Vitest equivalent with `runOnlyPendingTimers()` before `useRealTimers()`. Do not use `delay: null`; user-event explicitly warns that it causes unexpected behavior.
- `pointerEventsCheck` defaults to `EachApiCall`. On deeply nested trees it can dominate runtime; only lower it to `EachTarget` or `Never` when `pointer-events` behavior is not under test and profiling shows this is the hotspot.
- `queryBy*` is for absence only. Positive assertions should fail loudly with `getBy*` or `findBy*`, not with `Received: null`.
- Keep `waitFor` callbacks synchronous unless you truly need async work inside them; an async callback changes retry semantics because retries do not resume until that promise rejects.
- `screen.logTestingPlaygroundURL()` or role debugging is the first move when a semantic query surprises you; do that before reaching for test IDs.

## NEVER Do These

- NEVER fix an `act(...)` warning by wrapping `render`, `fireEvent`, or `user.click` in `act`, because RTL already wraps those paths. It is seductive because the warning disappears. The consequence is that the missing asynchronous branch stays untested and production can keep a loading or error state forever. Instead assert the post-async UI or disappearance state, and use manual `act` only for timer or promise advancement outside React.
- NEVER mock `fetch`, axios, or your API client in component tests because the stub will happily accept the wrong URL, method, or headers. It is seductive because the mock is local and fast. The consequence is endpoint drift that reaches production untouched. Instead intercept at the transport layer with MSW and `onUnhandledRequest: 'error'`.
- NEVER put side effects or multiple assertions inside `waitFor`, because the callback reruns on both polling and DOM mutations. It is seductive because it looks like an atomic "eventually" block. The consequence is repeated clicks or keypresses, timeout-only failures, and snapshots taken mid-transition. Instead perform the interaction once, then wait on one assertion.
- NEVER use `getByTestId`, `container.querySelector`, or component-instance queries when a semantic path exists, because you stop testing the accessibility tree. It is seductive because these selectors look stable against copy changes. The consequence is silent regressions in labels, roles, and focus behavior. Instead make the markup queryable by role and name or by label.
- NEVER add ARIA that duplicates or conflicts with native semantics, because it can change how assistive tech computes names and roles. It is seductive because it feels explicit. The consequence is tests that pass while screen readers announce the wrong thing. Instead prefer native elements and add ARIA only for widgets HTML cannot express.
- NEVER move shared UI setup into nested `describe` plus `beforeEach` plus mutable `let` state, because the reader must reconstruct the world before understanding any one test. It is seductive because it feels DRY. The consequence is order-coupled failures and helpers nobody wants to modify. Instead keep tests flat and use explicit override factories only after the rule of three.

## Failure Triage

- If a role query fails on one environment only, confirm whether the accessible name changed, whether the element is excluded from the accessibility tree, and whether you accidentally queried a superclass role like `checkbox` instead of the literal `switch`.
- If a disappearance wait throws immediately, the element was never present; assert its appearance or capture the element first, then wait for removal.
- If fake-timer tests hang only with `user-event`, wire `advanceTimers` before changing the app code.
- If a test gets faster only after swapping `getByRole` to `getByTestId`, you probably optimized around a DOM-size problem; scope the query or use a simpler semantic query rather than abandoning semantics.

## Freedom Calibration

- High freedom: which scenarios matter, which assertions best express user harm, when copy changes are product-significant, and where a file-local helper starts paying for itself.
- Low freedom: async primitive choice, timer restoration, MSW lifecycle, and the boundary between semantic queries and implementation selectors. Small deviations here create whole-suite flake or blind spots.

## Stop Condition

Keep the test only if both statements remain true after a refactor:
1. A real user-visible regression makes it fail.
2. A behavior-preserving implementation rewrite does not.
