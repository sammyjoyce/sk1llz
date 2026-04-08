---
name: dodds-testing-practices
description: Write JavaScript/TypeScript tests in the style of Kent C. Dodds (creator of React Testing Library, MSW advocate, Testing Trophy author). Use when writing or reviewing React/DOM tests with @testing-library, debugging act() warnings, picking between waitFor / findBy / getBy / queryBy, designing MSW network mocks, deciding between unit/integration/E2E, refactoring brittle Enzyme/snapshot tests, or whenever a test feels coupled to internals. Trigger keywords - testing-library, react-testing-library, RTL, screen, getByRole, findByRole, waitFor, userEvent, MSW, mock service worker, testing trophy, implementation details, AHA testing, shallow rendering, jest-dom, vitest, test user, integration test.
tags: testing, react, javascript, typescript, testing-library, msw, integration-testing, accessibility, jest, vitest
---

# Kent C. Dodds Testing Style⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍​​​‌​‌‌‌‍‌​‌‌​‌‌​‍​​​‌‌​​‌‍​‌​​‌​​​‍​​​​‌​​‌‍‌​​‌‌​​​⁠‍⁠

## The Mental Model: Two Users, Not Three

A UI component has exactly **two** users:

1. **The end user** — clicks, types, reads, hears (screen reader).
2. **The developer user** — imports it, passes props, renders it inside providers.

Every test you write must serve only these two. The moment a test knows about
state shape, hook names, internal methods, CSS classes, component display
names, or `data-testid` values that the end user can't see, you've invented a
third user — **the test user** — who pays no bills and breaks under refactors.

> "The more your tests resemble the way your software is used, the more
> confidence they can give you."

## The Two-Question Test for Any Assertion

Before writing or accepting any test, ask both:

1. **Will this test fail if I introduce a real user-visible bug?**
   (e.g. typo `onClick={tgogle}` instead of `toggle`)
2. **Will this test still pass after a backward-compatible refactor?**
   (e.g. renaming `toggle` → `handleButtonClick`, hooks → class, class → hooks,
   extracting a sub-component, swapping Redux for Context)

A good test answers **YES to both**. Tests that touch `.state()`, `.instance()`,
hook internals, snapshots of full component trees, or mock the component under
test fail one or both questions and are net-negative — they slow refactors
*and* miss real bugs.

## Query Decision Tree (memorize this — it eliminates 80% of bad tests)

```
Need to assert something EXISTS or interact with it?
├─ Will it appear later (async)?     → await screen.findByRole(...)
└─ It's already there?               → screen.getByRole(...)

Need to assert something does NOT exist?
└─ ONLY case for query*              → expect(screen.queryByRole(...)).not.toBeInTheDocument()

Need to wait for an arbitrary condition?
└─ await waitFor(() => expect(...))   ← exactly ONE assertion, NO side effects
```

**Why this matters (non-obvious):** `get*` and `find*` throw with a full
syntax-highlighted DOM dump on failure. `query*` only returns `null`, so an
assertion failure prints `"null is not in the document"` — useless. Always use
`get*`/`find*` for positive assertions; reserve `query*` exclusively for
non-existence checks.

**Why `findBy` over `waitFor(() => getBy)`:** they are equivalent under the
hood, but `findBy` produces a better error message and shorter call site.
Anyone writing `await waitFor(() => screen.getByRole(...))` is signalling they
don't know `findBy` exists.

## The Query Priority (the *only* order that matters)

1. `getByRole` with `{ name: /.../i }` — covers ~90% of cases. Implicit roles
   come free from semantic HTML; you almost never need to add `role=` yourself.
2. `getByLabelText` — form fields. Forces you to associate labels properly.
3. `getByPlaceholderText` / `getByText` / `getByDisplayValue` — when no role/label.
4. `getByAltText` / `getByTitle` — images, iframes, SVG.
5. `getByTestId` — last resort, for things end users genuinely cannot see
   (tracking pixels, complex SVG charts, container divs).

**Counterintuitive rule:** Querying by actual user-visible text — even
localized text — beats `data-testid`. If a copywriter changes "Sign up" to
"Get started" and your test breaks, that's a *feature* — you needed to know.
Run tests against the default locale.

### When `getByRole` fails — fallback ladder

1. **"Unable to find an accessible element with the role..."** — call
   `screen.logTestingPlaygroundURL()` (or `screen.debug()`) and read the
   "accessible roles" list RTL prints. Often the role is `'textbox'` not
   `'input'`, `'combobox'` not `'select'`, `'img'` only when `alt` is set.
2. **Element has the role but no accessible name** — add `htmlFor`/`id`
   association on the label, an `aria-labelledby`, or a visible text node.
   Do *not* paper over it with `data-testid`.
3. **Multiple elements match** — use `{ name: /.../i }` to disambiguate by
   accessible name. If two elements legitimately share a name, scope with
   `within(screen.getByRole('navigation'))` rather than `getAllByRole(...)[1]`.
4. **`<input>` has no role** — add `type="text"` (or `email`, `search`, etc.).
   Bare `<input>` has no implicit role; this is the #1 "but it's a textbox!" gotcha.
5. **Custom widget genuinely lacks semantics** — only now reach for
   `data-testid`, and file an accessibility bug against the component.

## Before Writing a Test, Ask Yourself

- **What does the end user see / hear / do?** That is your query and your assertion.
- **What does the developer user need to render this?** Wrap once in providers, not per-test.
- **Am I about to mock a child component?** First try rendering it. Mocks are
  for animation libraries, network, randomness, time, and modules with side
  effects on import.
- **Does this test know more than the two users?** If yes, delete the extra knowledge.

## The NEVER List (every item has a non-obvious failure mode)

NEVER use **Enzyme `shallow`** because it lets you refactor a real bug into
existence (typo on `onClick` still passes) while breaking tests on innocent
renames. Shallow rendering optimizes for the test user. **Instead:** mount with
RTL and `jest.mock()` heavy children explicitly when you need to isolate.

NEVER **wrap `render` or `fireEvent` in `act()`** to silence warnings. Both are
already wrapped in `act` internally — your wrapper does literally nothing. The
warning means you have un-awaited async state updates. **Instead:** use
`await userEvent...` or `await findBy...` or `await waitFor(...)` and the
warning disappears.

NEVER **put multiple assertions or any side effect inside `waitFor`**. The
callback runs both on an interval *and* on every DOM mutation, so side effects
fire 5–20 times. With multiple assertions, the test waits the full timeout
before reporting which one failed. **Instead:** one assertion per `waitFor`;
fire events *outside* the callback.

NEVER **snapshot a component tree**. Snapshots are pure implementation
details — full of prop names, component names, and structure that churn on
refactor. Reviewers become numb to "just press u" and snapshots become
write-only confidence theater. **Instead:** assert on specific user-visible
text/roles with `toHaveTextContent`, `toBeInTheDocument`, `toHaveAccessibleName`.

NEVER **mock `window.fetch` or your API client directly**. You'll hardcode
implementation details (URL paths, request shapes, response envelopes) into
every test, and a wrong-endpoint bug will pass. **Instead:** use **MSW**
(`msw/node`) so the same handlers serve tests, dev, and Storybook. See
`references/msw-patterns.md`.

NEVER **destructure queries from `render()`** (`const { getByRole } = render(...)`).
You'll constantly maintain the destructure as queries change. **Instead:** use
the global `screen` object — autocomplete works, no maintenance.

NEVER **call `cleanup` or `afterEach(cleanup)`**. RTL has done this
automatically since v9 (2019). Calling it manually is a tell that the codebase
or developer is years behind. **Just delete it.**

NEVER **add `role="button"` to a `<button>`** or `aria-label` to an element
that already has accessible text. Redundant ARIA actively *confuses* screen
readers and is the #1 mistake in "accessible" components. The implicit role is
already there. (Note: `<input>` only gets a role when `type` is set — that's
why `getByRole('textbox')` fails on bare `<input>`.)

NEVER **target 100% coverage on an application**. Diminishing returns kick in
hard around 70%. The remaining 30% drives engineers to test trivial code or
implementation details to chase the number. 100% is appropriate *only* for
small libraries (isolated, heavily reused, easy to maintain at 100%).

NEVER **re-run the login UI flow in every E2E test**. One UI test for the
login flow gives all the confidence you need; the other 99 should authenticate
via the same HTTP calls the app makes (or session injection / cookie set).
Repeating the UI flow is wasted minutes per test run with zero confidence gain.

NEVER **nest tests with `describe` + `beforeEach` to share state**. Shared
mutable state between tests creates ordering bugs and forces the reader to
trace upward through the file. **Instead:** use AHA Testing with a `setup()`
factory that returns everything (see `references/aha-testing.md`).

NEVER **use `fireEvent.change(input, ...)`** for typing. `userEvent.type` fires
`keydown` → `keypress` → `input` → `keyup` per character, plus focus events,
which is what real users do *and* what some libraries (Downshift, react-select,
contenteditable editors) actually listen for. `fireEvent.change` fires one
synthetic event and silently misbehaves with these libraries.

NEVER **call `userEvent.click(...)` without `setup()`** in user-event v14+.
You must do `const user = userEvent.setup()` once per test (before `render`),
then `await user.click(...)`. The legacy direct API still exists but lies about
async behavior. Always `await` user-event calls.

## Loading References (read only when scenario matches)

- **Fixing brittle tests, or unsure about a NEVER item's exact failure mode?**
  READ `references/anti-patterns.md` (14 patterns, severity-ranked, before/after).
- **Setting up MSW, mocking the network, or writing per-test edge-case handlers?**
  READ `references/msw-patterns.md` (server lifecycle, colocation, gotchas).
- **Test file getting repetitive, or you have ≥3 similar tests in one file?**
  READ `references/aha-testing.md` (Test Object Factory, `renderFoo()`, the
  ANA↔DRY spectrum, when to use `it.each` instead).
- **Do NOT load any reference for one-off questions about query priority, the
  NEVER list, or the two-user model — those live here in SKILL.md.**

## Signature Heuristics

- Default to integration tests rendering the real tree with real providers;
  mock only network (MSW), animation, time, and randomness.
- Ship a test only if it answers YES to both questions in §"The Two-Question
  Test." Otherwise rewrite or delete it.
- Confidence is the goal, not coverage. 60% coverage of integration tests
  beats 95% coverage of `state()` assertions.
