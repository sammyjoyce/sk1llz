# MSW (Mock Service Worker) Patterns⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌‌‌‌‌‍​‌‌​​‌‌‌‍​​‌​​​​‌‍‌‌​​​​‌‌‍​​​​‌​​‌‍‌​​​​​‌​⁠‍⁠

Kent's "Stop Mocking Fetch" thesis: mocking `window.fetch` (or your API
client) hardcodes implementation details (URLs, headers, response shape) into
every test. If you call the wrong endpoint, your test still passes — you only
discover the bug in production. MSW intercepts at the network boundary, so
"wrong endpoint" → handler not invoked → test fails correctly.

The same MSW handlers can run in Node (tests), the Service Worker (browser
dev), and Storybook. One source of mock truth.

---

## Layer 1 — Shared handlers (one file, used everywhere)

```js
// test/server-handlers.js   (also imported by dev / Storybook setup)
import { http, HttpResponse } from 'msw'  // MSW v2 API
import { db } from './db'                 // in-memory db, builders, or static fixtures

export const handlers = [
  http.get('/api/user/:id', ({ params }) => {
    const user = db.user.findById(params.id)
    if (!user) return new HttpResponse(null, { status: 404 })
    return HttpResponse.json(user)
  }),

  http.post('/api/login', async ({ request }) => {
    const { email, password } = await request.json()
    const user = db.user.authenticate(email, password)
    if (!user) {
      return HttpResponse.json({ error: 'Invalid credentials' }, { status: 401 })
    }
    return HttpResponse.json({ token: db.token.create(user.id), user })
  }),
]
```

**Critical rule:** handlers represent the **happy path only**. Edge cases live
in the test that needs them (Layer 3).

> MSW v1 used `rest.get(...)` with `(req, res, ctx) => res(ctx.json(...))`.
> MSW v2 (current) uses `http.get(...)` with `() => HttpResponse.json(...)`.
> Pin the version in package.json and don't mix the two styles.

---

## Layer 2 — Server lifecycle (one-time global setup)

```js
// test/setup.js — referenced from setupFilesAfterEach (Jest) or setupFiles (Vitest)
import { setupServer } from 'msw/node'
import { handlers } from './server-handlers'

export const server = setupServer(...handlers)

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))
//                                ^^^^^^^^^^^^^^^^^^^^^^^^
// ALWAYS use 'error'. The default 'warn' lets typo'd URLs slip through —
// the test passes silently because no handler matched and fetch resolves
// to nothing useful. 'error' surfaces the bug immediately.

afterEach(() => server.resetHandlers())  // critical for test isolation
afterAll(() => server.close())
```

---

## Layer 3 — Per-test runtime overrides (edge cases)

```js
import { server } from 'test/setup'
import { http, HttpResponse } from 'msw'

test('shows server error if checkout fails', async () => {
  server.use(
    http.post('/api/checkout', () =>
      HttpResponse.json({ message: 'card declined' }, { status: 402 })
    )
  )

  const user = userEvent.setup()
  render(<Checkout />)
  await user.click(screen.getByRole('button', { name: /confirm/i }))

  expect(await screen.findByRole('alert')).toHaveTextContent(/card declined/i)
})
// afterEach(() => server.resetHandlers()) above wipes this override
// so the next test gets the happy-path handler again. NEVER skip resetHandlers.
```

This is the colocation answer: "happy path" stays in `server-handlers.js`,
"this specific test needs failure" stays in the test that needs it.

---

## The test-data-bot / faker factory pattern

Kent's preferred way to seed inputs without coupling tests to literal values:

```js
import { build, fake } from '@jackfranklin/test-data-bot'

const buildLoginForm = build({
  fields: {
    username: fake((f) => f.internet.userName()),
    password: fake((f) => f.internet.password()),
  },
})

test('logging in displays the username', async () => {
  const { username, password } = buildLoginForm()
  const user = userEvent.setup()
  render(<App />, { route: '/login' })

  await user.type(screen.getByLabelText(/username/i), username)
  await user.type(screen.getByLabelText(/password/i), password)
  await user.click(screen.getByRole('button', { name: /submit/i }))

  expect(await screen.findByText(username)).toBeInTheDocument()
})
```

**Why builders beat fixtures:** with a literal `'alice'` test, a copy-paste
typo can make two tests "share" a username and pass for the wrong reason.
Random data per test guarantees true isolation, and override syntax
(`buildLoginForm({ overrides: { username: 'alice' } })`) handles the cases
where you need a specific value.

---

## Common MSW gotchas

**Gotcha 1 — `fetch` polyfill in Jest jsdom.** jsdom historically lacked
`fetch`. Solutions: install `whatwg-fetch` *or* upgrade to Node 18+ with
`testEnvironmentOptions.customExportConditions: ['']` *or* switch to Vitest.
Without this, MSW handlers exist but `fetch is not defined` in your component.

**Gotcha 2 — `onUnhandledRequest: 'warn'` (the default) is dangerous.** Always
set `'error'`. A typo URL produces zero output otherwise.

**Gotcha 3 — handlers leak between tests.** The `server.resetHandlers()` line
in `afterEach` is non-negotiable. Every dev who removes it will spend a day
debugging "why is test B failing only when test A runs first?"

**Gotcha 4 — `server.use()` ordering.** Runtime overrides take priority over
shared handlers, but they're added to a stack. If you call `server.use()`
twice in one test for the same route, the second wins. Reset clears the stack.

**Gotcha 5 — MSW does not intercept Node `http.request` directly until v2.**
If you use `node-fetch` v2 or `axios` with the http adapter, you may need
`msw/node`'s interceptor configuration. v2 uses `@mswjs/interceptors` which
covers `http`, `https`, `XMLHttpRequest`, and `fetch`.
