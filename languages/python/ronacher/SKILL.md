---
name: ronacher-pragmatic-design
description: "Design Python libraries, Flask/Werkzeug/Jinja/Click code, and extension APIs in Armin Ronacher's style: explicit application objects, composable primitives, sharp but documented escape hatches, and minimal magic. Use when shaping framework or library interfaces, Flask extensions, request-context code, blueprints vs WSGI composition, Jinja environments or filters, or Click CLIs. Triggers: Flask, Werkzeug, Jinja, Click, LocalProxy, current_app, app factory, blueprint, WSGI middleware, autoescape, StrictUndefined, init_app."
tags: python, flask, werkzeug, jinja, click, wsgi, app-factory, extension-design, api-design, library-design
---

# Ronacher Pragmatic Design⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌‌​​​‍​‌​‌‌​‌‌‍​‌​‌​​‌​‍‌‌​​​‌​‌‍​​​​‌​​‌‍‌​‌​​​‌​⁠‍⁠

## Start Here

- If the task changes a public or reusable API, read everything.
- If it only changes application code, read `Library vs Application`, `Flask and Werkzeug`, and `NEVER`. Skip `Click` and `Jinja` unless those surfaces are involved.
- If it only touches templates or filters, read `Jinja` and `NEVER`. Skip `Click`.
- If it only touches CLI behavior, read `Click` and `NEVER`. Skip `Jinja`.
- This skill is intentionally self-contained. Do not load generic Flask, Jinja, or Click tutorials unless you are checking an exact API signature.

## Library vs Application

Before writing code, ask yourself:

- Is this disposable product code or a reusable surface with a multi-year compatibility half-life?
- If users cannot reach a low-level primitive directly, what hack will they invent?
- If a feature relies on hidden naming rules, frame inspection, cwd, or import side effects, how will someone debug it at 2 a.m.?

Ronacher's split is sharp:

- In applications, local ugliness is acceptable when it ships value faster.
- In libraries and frameworks, clever convenience becomes permanent support debt.
- Expose a small primitive plus a boring composition story instead of a magical shortcut that users cannot grep, test, or override.

## Decision Trees

### New Flask structure

- Need shared config and one lifecycle? Use blueprints.
- Need separate configs, separate teardown, or the ability to discard one app independently? Compose multiple WSGI apps or middleware instead of overloading blueprints.
- Need reusable setup across apps? Prefer `init_app` on an extension or a blueprint that records operations.

### Request-scoped state

- One contextual value? Prefer a `ContextVar` plus `LocalProxy` directly.
- A namespace or stack only if truly necessary. `Local` and `LocalStack` copy mutable data in nested contexts and cost more.
- Need to hand data to a signal, thread, or library that does not understand proxies? Resolve once with `_get_current_object()`.

### Jinja safety

- Missing data is a bug? Use `StrictUndefined`.
- Missing data should chain for author ergonomics? Use `ChainableUndefined`, but only with a clear reason.
- Rendering untrusted templates? Sandbox plus narrow input plus CPU, memory, and output limits. Sandbox alone is not the security boundary.

### Click parsing

- Early-exit flag like `--version` or `--help`? Make it eager.
- Derived defaults or validation depend on other params? Remember missing params fire last, repeated params fire at the position of first occurrence, and user order normally wins.
- Wildcard-driven CLI may receive zero files? Prefer graceful no-op over required positional args unless the command is destructive or ambiguous.

## Flask and Werkzeug

Before touching Flask architecture, ask yourself:

- Does this code run during setup, request handling, or CLI execution?
- Does it need one app instance or many?
- Is the state per app, per request, per CLI invocation, or truly process-global?

Use these rules:

- Keep the explicit application object. Its value is not only testability; it also anchors resource loading via the package name. Cwd-based template or static lookup is unreliable because cwd is process-wide and may not point at the app at all.
- Finish all setup before the first request. Late calls that mutate routes, blueprints, config, `app.jinja_env`, JSON providers, or extension wiring create worker divergence. Flask detects only some of these mistakes.
- In extensions, do not retain `self.app`. Use `init_app(app)`, store per-app state in `app.extensions[...]`, and use `current_app` at runtime. This preserves factories, avoids circular imports, and lets one extension instance serve multiple apps.
- Separate configuration layers deliberately:
  - `app.config` for per-deployment values.
  - `__init__` args for per-extension construction choices.
  - instance attributes or decorators for ergonomic registration.
  - class attributes or subclass hooks for advanced override points.
- `g` is shared namespace for one app context, not a persistence layer. Prefix internal names or use a namespace object to avoid collisions with user data.
- Choose teardown scope precisely:
  - `teardown_appcontext` for resources valid in requests and CLI commands.
  - `teardown_request` only for request-only data.
  - Teardown callbacks must survive partial dispatch; they cannot assume `before_request` or the view ran.
- If you see `Working outside of application context` during setup, a manual `with app.app_context():` block is fine. If you see it somewhere else, that is usually a design smell: move the work to a view, CLI command, or explicit boundary instead of pushing context deeper into helpers.
- If you need to alter a response before one exists, do not contort control flow. Use `after_this_request()` instead of smuggling response state through globals.
- Blueprints are recorded operations, not pluggable apps. They can be registered multiple times only if written for it; otherwise endpoint names and one-time setup assumptions collide. They cannot be unregistered without rebuilding the app object.
- Flask routing orders rules by complexity. If your design depends on import order or decorator order across modules, the abstraction boundary is probably wrong.
- Flask `async` views preserve extension compatibility by running the coroutine on a separate thread. Treat them as an integration convenience, not as ASGI-style throughput or latency behavior.
- Signal receivers should accept `**extra`, and signal senders should pass real objects such as `current_app._get_current_object()`. Flask may add new kwargs later, and proxy senders break identity-sensitive consumers.

## Context Locals and Proxies

Before introducing a proxy, ask yourself:

- Am I choosing a proxy because passing the object is noisy, or because the lifetime boundary is genuinely contextual?
- Will this ever cross into async tasks, background threads, signal handlers, or foreign libraries?

Use these rules:

- Prefer direct `ContextVar` proxies over `Local` or `LocalStack` unless you truly need a mutable namespace or stack.
- Create context vars at module global scope. Creating them inside request-time helpers can interfere with garbage collection.
- Cache `_get_current_object()` once if you will access the proxied object repeatedly in a hot function.
- Treat truthiness carefully: `bool(proxy)` is `False` when unbound, which can hide context bugs behind innocent `if request:` checks.
- Pass real objects, not proxies, across boundaries such as signals, background threads, or libraries that inspect type or identity.

## Jinja

Before changing template behavior, ask yourself:

- Is this value common to every render of every template, or only this render?
- Am I changing environment behavior before the first template load or after caches already exist?
- Does this filter need the environment, the eval context, or the full render context?

Use these rules:

- Configure the environment once during startup. Mutating filters, tests, globals, or policies after the first template load causes surprising and sometimes undefined behavior because state is shared and cached.
- Keep per-render data in `render(...)`, not `Environment.globals`. Globals are for truly universal values. Template globals after load are also unsafe to mutate.
- Use `select_autoescape(['html', 'htm', 'xml'])` as the baseline. If you render HTML from strings, enable `default_for_string=True`, and make sure any custom autoescape chooser handles `None` for string-based templates.
- For filters that emit HTML, inspect `eval_ctx.autoescape` via `pass_eval_context`, not `env.autoescape`. Autoescape is computed per template, so environment-level checks are wrong under overrides.
- Do not mutate the evaluation context at runtime. If you need runtime policy changes, do it through extension nodes, not ad hoc state changes.
- Default `Undefined` can quietly print as empty string and pass boolean checks. That is friendly for authoring and terrible for bug hunting. Reach for `StrictUndefined` whenever missing data should fail fast.
- If you need custom undefined objects, construct them with `environment.undefined(...)` and pass `obj` and `name` when known so the eventual error message stays useful.
- Sandboxed Jinja still lets users exhaust CPU or memory with small templates that expand massively. Restrict data, mark side-effectful callables unsafe, prefer `ImmutableSandboxedEnvironment` when mutation matters, and enforce resource limits outside Jinja.

## Click

Before designing a CLI surface, ask yourself:

- Is this command meant to be composed in shell aliases or scripts?
- Can empty input be a valid no-op?
- Does my callback logic depend on decorator order instead of parser order?

Use these rules:

- Prefer options for most input except subcommands, URLs, and files. Ronacher-style CLIs optimize for composability, not novelty.
- Avoid required positional arguments when empty wildcard expansion should safely do nothing.
- Respect callback evaluation rules:
  - normal params fire in user-supplied order;
  - eager params fire before non-eager params;
  - repeated params fire based on the first occurrence;
  - missing params still fire, but at the end.
- Even when an option is not declared as multi-value, Click may preserve the position of the first appearance while taking the last value. That behavior exists to keep shell aliases composable; do not build validation logic that assumes duplicates are impossible.
- If you want shell completion, remember it only works when installed as an entry point, not when invoked through `python script.py`.

## NEVER

- NEVER invent implicit app-singleton magic because it looks concise. It blocks multiple app instances, weakens test isolation, and removes the package anchor Flask uses for resource lookup. Instead keep an explicit app object or factory.
- NEVER base resource discovery on cwd because it feels convention-friendly. Cwd is process-wide and breaks under multi-app servers and alternate launchers. Instead use the app package name or `PackageLoader`.
- NEVER store `self.app = app` in an extension because it is seductive during prototyping. It quietly breaks factories, multi-app use, and tests. Instead keep per-app state in `app.extensions` and use `current_app`.
- NEVER mutate routes, config, blueprints, JSON providers, or `app.jinja_env` after serving starts because only some workers will see the change. Instead do all registration in the factory or `init_app`.
- NEVER use `g` as durable storage because its name feels global. It dies with the app context and shares one namespace with the whole app. Instead use `session`, a database, or a namespaced `g` key for request and CLI caches only.
- NEVER pass `current_app`, `request`, or any `LocalProxy` as a signal sender or background-task payload because the proxy is not the real object and the context may be gone. Instead pass `_get_current_object()` or plain data.
- NEVER check `env.autoescape` inside a filter that emits HTML because that value is not the per-template truth. Instead use `pass_eval_context` and branch on `eval_ctx.autoescape`.
- NEVER treat `SandboxedEnvironment` as a complete trust boundary because side-effectful objects and output amplification still escape the box. Instead narrow the context and add external resource limits.
- NEVER hide essential primitives behind nested closures or naming tricks because users will reimplement them badly once they need the low-level piece. Instead expose the boring primitive and keep the sugar thin.
- NEVER assume teardown hooks run after the full request pipeline because they feel like "after request". They run when contexts pop, even after partial failure or manual context pushes. Instead write teardown code that is idempotent and independent.

## Fallbacks

- If a Flask design starts fighting factories, stop and move the work to `create_app()` or `init_app()` before adding more indirection.
- If context-local behavior becomes hard to reason about, replace proxies with explicit parameter passing at the boundary and keep proxies only at the top edge.
- If Jinja safety policy is unclear, bias toward `StrictUndefined`, autoescape on, narrow globals, and explicit `Markup` only where justified.
- If Click callback interactions become tricky, simplify the surface area before adding more callbacks; parser-order bugs are cheaper to prevent than to debug.
- If a feature requires large-app async semantics, separate app lifecycles, or cross-request mutation, treat that as evidence you are leaving Flask's sweet spot rather than a cue to add more magic.
