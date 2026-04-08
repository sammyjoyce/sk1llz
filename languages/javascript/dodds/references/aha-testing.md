# AHA Testing — Avoid Hasty Abstraction

Most test files sit at one of two broken extremes:

| Style | What it looks like | Failure mode |
|---|---|---|
| **ANA** (Absolutely No Abstraction) | Every test has a 60-line literal `req`/`res` mock copy-pasted | "Where's Wally" — diffs between two tests are buried in 200 lines of identical setup |
| **DRY** (Don't Repeat Yourself) | `describe` nesting, `beforeEach` mutations, shared `let` variables | Reader must trace upward through hooks to understand any one test; ordering bugs; hidden state |

The middle is **AHA**: extract a `setup()` factory that takes overrides. Each
test makes its *unique* inputs visible right next to the assertion.

---

## The Test Object Factory pattern

```js
function setup(overrides = {}) {
  const req = {
    user: {
      guid: '0336397b-...',
      name: { first: 'Francine', last: 'Oconnor' },
      latitude: 51.507351,    // London
      longitude: -0.127758,
      ...(overrides.user ?? {}),
    },
    params: { bucket: 'photography' },
    header: (name) => ({ Authorization: 'Bearer TEST_TOKEN' }[name]),
    ...overrides,
  }
  const res = {
    json: jest.fn(),
    sendStatus: jest.fn(),
    locals: { content: {} },
  }
  return { req, res, next: jest.fn() }
}

test('lists posts for the logged-in user (London)', async () => {
  const { req, res, next } = setup()
  await blogPostController.loadBlogPosts(req, res, next)
  expect(res.json).toHaveBeenCalledWith({
    posts: expect.arrayContaining([
      expect.objectContaining({ title: 'Test Post 1' }),
    ]),
  })
})

test('returns empty list when user is in Shanghai', async () => {
  const { req, res, next } = setup({
    user: { latitude: 31.230416, longitude: 121.473701 },
  })
  await blogPostController.loadBlogPosts(req, res, next)
  expect(res.json).toHaveBeenCalledWith({ posts: [] })
})
```

**The win:** with one glance you know test 2 is about Shanghai. With ANA you
would scroll 60 lines to find the lat/long differs by a fraction.

---

## React equivalent: `renderFoo()`

```js
function renderLoginForm(props) {
  const utils = render(<LoginForm {...props} />)
  const user = userEvent.setup()
  const usernameInput = screen.getByLabelText(/username/i)
  const passwordInput = screen.getByLabelText(/password/i)
  const submitButton = screen.getByRole('button', { name: /submit/i })

  return {
    ...utils,
    user,
    usernameInput,
    passwordInput,
    submitButton,
    fillForm: async ({ username, password }) => {
      await user.type(usernameInput, username)
      await user.type(passwordInput, password)
    },
    submit: () => user.click(submitButton),
  }
}

test('submit calls the submit handler with form data', async () => {
  const handleSubmit = jest.fn()
  const { fillForm, submit } = renderLoginForm({ onSubmit: handleSubmit })
  await fillForm({ username: 'alice', password: 'p@ssw0rd' })
  await submit()
  expect(handleSubmit).toHaveBeenCalledWith({ username: 'alice', password: 'p@ssw0rd' })
})
```

---

## When NOT to abstract — the rule of three

Kent's threshold: **wait until you have at least 3 similar tests** before
extracting `setup()` or `renderFoo()`. With 1-2 tests, the abstraction obscures
more than it saves. With ≥3, the duplication actively hides differences.

You can also keep the abstraction *inside the file* — don't immediately hoist
to `test-utils.js` until a second file needs it.

---

## Avoid nesting (`describe` + `beforeEach`)

```js
// ❌ DRY-extreme
describe('UserSettings', () => {
  let user, render
  beforeEach(() => {
    user = userEvent.setup()
    render = (props) => rtlRender(<UserSettings {...props} />)
  })
  describe('when admin', () => {
    let result
    beforeEach(() => { result = render({ role: 'admin' }) })
    test('shows delete', () => { expect(screen.getByRole('button', { name: /delete/i })).toBeVisible() })
  })
})

// ✅ flat
test('admin sees the delete button', () => {
  const { user } = setup({ role: 'admin' })
  expect(screen.getByRole('button', { name: /delete/i })).toBeVisible()
})
```

**Why flat wins:**
1. The reader never has to scroll up to find what `result` and `user` are.
2. No shared mutable state between tests — eliminates ordering bugs.
3. `setup()` is explicit per-test; nothing is "magic" from a hook.
4. Tests can be moved or deleted without thinking about hook side effects.

---

## When AHA is wrong: parameterized tests

For pure functions with many input/output cases, use `it.each` / `test.each`
or `jest-in-case` instead of `setup()`:

```js
import cases from 'jest-in-case'
import fizzbuzz from '../fizzbuzz'

cases(
  'fizzbuzz',
  ({ input, output }) => expect(fizzbuzz(input)).toBe(output),
  [
    { name: '1 → 1',         input: 1,  output: '1' },
    { name: '3 → Fizz',      input: 3,  output: 'Fizz' },
    { name: '5 → Buzz',      input: 5,  output: 'Buzz' },
    { name: '15 → FizzBuzz', input: 15, output: 'FizzBuzz' },
  ],
)
```

`jest-in-case` produces a separate test name per case (so failures point at
exactly the row that broke), which is its advantage over a literal
`forEach` + `test()`.
