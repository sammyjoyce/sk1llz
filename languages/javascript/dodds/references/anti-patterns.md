# React Testing Library Anti-Patterns (with severity)⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​‌‌​‌‌‌‍​​​‌​​​‌‍‌​‌​‌​‌​‍‌‌‌‌‌‌​‌‍​​​​‌​‌​‍​​​​​‌‌‌⁠‍⁠

The 14 patterns below are the ones Kent Dodds repeatedly calls out. Each lists
the *seductive reason* people write it, the *concrete consequence*, and the
fix. Severity follows Kent's own labels: **HIGH** = you are missing confidence
or shipping bugs; **MED** = bugs/wasted work likely; **LOW** = stylistic.

---

## 1. HIGH — Querying with `getByTestId` when a role exists

```js
// ❌
screen.getByTestId('username')

// ✅
screen.getByRole('textbox', { name: /username/i })
```

**Seductive because:** test IDs feel "stable" — they don't change with copy.
**Consequence:** the test no longer verifies that the input is reachable by
assistive tech. A regression that removes the `<label>` association ships
silently. You also accept worse test IDs as your team's default forever.
**Fix:** add `<label htmlFor>` + `id` + `type="text"` to the input. The role
appears for free.

> Note: `<input>` without a `type` attribute has **no implicit role** —
> `getByRole('textbox')` will fail. This is one of the most common "but it
> should work!" gotchas.

---

## 2. HIGH — Using `query*` for positive assertions

```js
// ❌
expect(screen.queryByRole('alert')).toBeInTheDocument()

// ✅
expect(screen.getByRole('alert')).toBeInTheDocument()
expect(screen.queryByRole('alert')).not.toBeInTheDocument()  // only valid query* use
```

**Seductive because:** the names look symmetrical — `queryBy` for assertions,
`getBy` for "fetch and use."
**Consequence:** when the element is missing, `queryBy` returns `null` and the
test prints `Received: null`. With `getBy`, RTL throws and dumps the **full
syntax-highlighted DOM** so you can see what *did* render. You lose hours of
debugging time per failure.
**Fix:** `query*` is *only* for asserting non-existence.

---

## 3. HIGH — `waitFor` instead of `findBy`

```js
// ❌
const btn = await waitFor(() => screen.getByRole('button', { name: /submit/i }))

// ✅
const btn = await screen.findByRole('button', { name: /submit/i })
```

**Why:** `findBy` *is* `waitFor` + `getBy` internally, but with much better
errors and intent. Mixing the two signals you don't know `findBy` exists.

---

## 4. HIGH — Multiple assertions inside `waitFor`

```js
// ❌
await waitFor(() => {
  expect(window.fetch).toHaveBeenCalledWith('/api/foo')
  expect(window.fetch).toHaveBeenCalledTimes(1)
})

// ✅
await waitFor(() => expect(window.fetch).toHaveBeenCalledWith('/api/foo'))
expect(window.fetch).toHaveBeenCalledTimes(1)
```

**Consequence:** if assertion 2 fails, `waitFor` retries until full timeout
(default 1000ms, often configured to 5000ms) before reporting. Failures take
~5s instead of being instant. Worse: if you put a snapshot inside `waitFor`,
the snapshot may be taken mid-update and pass *or* fail nondeterministically.

---

## 5. HIGH — Side effects inside `waitFor`

```js
// ❌
await waitFor(() => {
  fireEvent.keyDown(input, { key: 'ArrowDown' })
  expect(screen.getAllByRole('listitem')).toHaveLength(3)
})

// ✅
fireEvent.keyDown(input, { key: 'ArrowDown' })
await waitFor(() => {
  expect(screen.getAllByRole('listitem')).toHaveLength(3)
})
```

**Why this is so bad:** the `waitFor` callback fires both on a polling
interval **and** on every DOM mutation. So the `keyDown` runs 5–20 times
before the assertion passes. Random behavior, flaky tests, hidden bugs.

---

## 6. HIGH — Testing implementation details

```js
// ❌
const wrapper = mount(<Counter />)
expect(wrapper.state('count')).toBe(0)
wrapper.instance().increment()
expect(wrapper.state('count')).toBe(1)
```

**The two-question test:** Can I introduce a bug this won't catch? **Yes** —
typo `onClick={incrment}` still passes. Can a refactor break it? **Yes** —
renaming `increment` → `handleClick` breaks it. Both questions fail. Delete it.

```js
// ✅
const user = userEvent.setup()
render(<Counter />)
const button = screen.getByRole('button', { name: '0' })
await user.click(button)
expect(screen.getByRole('button', { name: '1' })).toBeInTheDocument()
```

---

## 7. HIGH — Adding ARIA "willy nilly"

```js
// ❌
<button role="button" aria-label="Submit">Submit</button>
<div role="button" tabIndex={0} onClick={...}>Click</div>  // also bad

// ✅
<button>Submit</button>
```

**Consequence:** redundant ARIA *overrides* the implicit role and confuses
screen readers (NVDA may announce "button button" or skip the visible text).
If you can use `<button>`, you must. ARIA is for cases where semantic HTML
genuinely can't express the widget (custom comboboxes, treeviews, etc.) and
even then, follow WAI-ARIA Authoring Practices examples — don't invent.

---

## 8. MED — Wrapping things in `act()` to silence warnings

```js
// ❌
act(() => { render(<Example />) })
act(() => { fireEvent.click(button) })

// ✅
render(<Example />)
fireEvent.click(button)
// If you still see a warning, it means async work is unawaited:
await screen.findByText(/loaded/i)
```

**Why:** RTL's `render`, `fireEvent`, and `userEvent` are **already wrapped in
`act` internally**. Wrapping them again is a no-op. The warning is telling you
"I detected a state update outside of `act`" — almost always an unawaited
promise (`fetch` resolving, `setTimeout` callback, microtask). The fix is to
`await` the assertion that depends on the result.

---

## 9. MED — `fireEvent.change` for typing

```js
// ❌
fireEvent.change(input, { target: { value: 'hello' } })

// ✅
const user = userEvent.setup()
await user.type(input, 'hello')
```

**Why:** real users fire `keydown`, `keypress`, `input`, `keyup` per
character, plus focus events. `fireEvent.change` fires one synthetic
React-only event. Libraries like **Downshift, react-select, Slate, Lexical,
contenteditable, IMEs** listen for the keyboard events — your tests will pass
while the real app silently breaks.

---

## 10. MED — Not using `screen` (destructuring from `render`)

```js
// ❌
const { getByRole, getByText, queryByLabelText } = render(<App />)
getByRole('button')

// ✅
render(<App />)
screen.getByRole('button')
```

**Why:** `screen` was added in DOM Testing Library 6.11. Using it means you
never maintain destructure lists, autocomplete works on `screen.`, and
`screen.debug()` is always available. The only legitimate reason to destructure
is if you need `container` or `baseElement` — and you almost certainly don't.

---

## 11. MED — Using `cleanup` manually

```js
// ❌
import { render, cleanup } from '@testing-library/react'
afterEach(cleanup)

// ✅
import { render } from '@testing-library/react'
// nothing — cleanup is automatic in Jest, Vitest, Mocha (since RTL v9, 2019)
```

If you see this in a codebase, the codebase or its author has not been updated
in 5+ years. Delete it everywhere and remove the `cleanup` import.

---

## 12. MED — Snapshot testing components

```js
// ❌
expect(container).toMatchSnapshot()
```

**Why:** snapshots are full of class names, prop names, child component names,
and DOM structure. Every refactor produces a noisy diff that looks identical
to the source diff. Reviewers train themselves to "press u" without reading.
Snapshot becomes write-only confidence theater that forbids refactoring.
**Acceptable snapshots:** small, focused, *deterministic* values — error
messages, generated CSS strings, schema introspection. Use `toMatchInlineSnapshot`
so the snapshot lives next to the assertion.

---

## 13. LOW — `getBy*` as the assertion itself

```js
// ⚠️ works but communicates intent poorly
screen.getByRole('alert')

// ✅ explicit
expect(screen.getByRole('alert')).toBeInTheDocument()
```

**Why Kent prefers the assertion form:** future readers may delete what looks
like a "stale orphan query." The `expect(...)` makes intent explicit. Both
work — `getBy` would throw on missing element anyway — so this is style.

---

## 14. MED — Not installing the ESLint plugins

```bash
npm i -D eslint-plugin-testing-library eslint-plugin-jest-dom
```

These plugins catch ~10 of the patterns above automatically. `create-react-app`
ships them; everyone else should install them. Rules to enable as **error**:
`testing-library/no-debugging-utils`, `testing-library/no-node-access`,
`testing-library/no-container`, `testing-library/prefer-screen-queries`,
`testing-library/prefer-find-by`, `testing-library/no-wait-for-multiple-assertions`,
`testing-library/no-wait-for-side-effects`, `jest-dom/prefer-in-document`,
`jest-dom/prefer-to-have-text-content`.
