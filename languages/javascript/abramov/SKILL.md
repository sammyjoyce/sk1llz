---
name: abramov-state-composition
description: Write React and Redux-flavored JavaScript in Dan Abramov's style: minimal state, honest effects, composition before memoization, reducers for event logs, and boundaries that survive refactors. Use when refactoring hooks, placing state, fixing stale closures, removing prop-state sync bugs, deciding local state vs reducer vs Redux, or untangling render-performance problems. Triggers: react, hooks, useEffect, useReducer, composition, memo, stale closure, derived state, Redux, normalized state.
---

# Abramov State Composition

This is a philosophy skill, not a snippet catalog. Use it to choose better boundaries before writing code.

## Load Boundary

- Use this skill for React state shape, effect semantics, reducer boundaries, Redux placement, stale closures, keyed resets, and render-performance triage.
- Before touching timers, subscriptions, or custom effect hooks, READ:
  - `https://overreacted.io/a-complete-guide-to-useeffect/`
  - `https://overreacted.io/making-setinterval-declarative-with-react-hooks/`
- Before moving data into Redux or flattening store shape, READ:
  - `https://redux.js.org/faq/organizing-state/`
  - `https://redux.js.org/usage/structuring-reducers/normalizing-state-shape`
- Do NOT open generic Hooks tutorials for this work. They usually reintroduce lifecycle folklore and lower the signal.

## Core Model

- Effects are for synchronizing React with systems outside React. If the logic exists only to derive UI from props/state, keep it in render. If it exists because a user acted, keep it in the event handler.
- Every render owns its own props, state, handlers, and effects. Stale closures are usually not a React problem; they mean you chose the wrong boundary between render-time data and a long-lived imperative API.
- State should store facts, not convenience copies. Store ids, status enums, and committed data; derive labels, counts, filtered lists, and selected objects during render or in selectors.
- Composition is the first performance tool. Memoization is boundary polish, not the primary architecture.

## Before You Change Anything, Ask

- Before adding state: can this value be derived from existing props/state during render?
- Before adding an effect: what external system am I synchronizing with?
- Before adding a reducer: am I recording what happened, or am I hiding messy `setState` calls?
- Before adding Redux: who besides this component needs the value, and do they need every intermediate edit?
- Before adding a ref: am I bridging an imperative API, or am I suppressing an honest dependency?
- Before adding `memo`, `useMemo`, or `useCallback`: can I move the changing state lower, or move the expensive subtree higher as children?

## Decision Tree

1. State lives in one component and dies with it: use local `useState`.
2. Multiple handlers update the same domain object, or you need an action log you can reason about: use `useReducer`.
3. Other screens need the same committed data, selectors matter, or time-travel/cache/hot-reload persistence matters: use Redux.
4. The user is editing text and nobody else needs every keystroke: keep it local and commit on blur, save, or submit.
5. A prop change means "this is a different entity now": key a child boundary and let React reset the subtree.
6. Only one field must be adjusted after an input changes: derive it or adjust during render; do not render stale UI and then "fix" it in an effect.
7. Performance is bad: profile in production first. Dev builds can be roughly an order of magnitude slower, so treat dev-only slowness as a hint, not proof.

## Procedures That Matter

### When an effect dependency feels "annoying"

1. Make the dependency list honest first.
2. If that re-runs the effect too often, reduce dependencies by changing code, not by lying:
   - replace read-modify-write state updates with updater functions
   - move helpers used only by the effect inside the effect
   - hoist helpers that need no render-scope data
   - stabilize true dependencies with `useCallback`
   - if the effect should describe what happened while state logic stays fresh, dispatch to a reducer
3. Only use a ref bridge when the external API itself is long-lived and imperative, like `setInterval`, subscriptions, or third-party listeners.

### When state keeps drifting out of sync

1. Remove duplicated objects from state; keep the canonical collection and store an id.
2. Replace parallel booleans with a single status field.
3. If a prop only seeds local state once, rename it to `initialX` or `defaultX` so ignoring later updates is explicit.
4. If later prop updates must win, make the component controlled instead of syncing props into state.

### When a reducer is the right move

- Reach for `useReducer` when updates are better described as events than assignments.
- Keep actions at interaction level. "user submitted", "tick", and "todo toggled" are good. "setFieldA" + "setFieldB" + "setFieldC" is usually a smell.
- If an effect should stay mounted but next state depends on fresh props, `useReducer` is the "cheat mode": dispatch from the effect, let the reducer compute during the next render, and keep the effect decoupled from changing values.
- Do not make reducers inside components your default style. That pattern disables some optimizations; use it when preserving a long-lived effect boundary is worth that trade-off.

### When performance hurts

- First fix state placement. A slow subtree under fast-changing state is a boundary bug before it is a memoization bug.
- Prefer "move state down" or "lift stable content up as children" before adding `memo`.
- In Redux lists, select ids in the parent and let items select themselves. Passing full objects through a large list widens re-render blast radius.
- If you use memoized selectors, verify the memoizer and cache policy instead of assuming reuse. Parameterized selectors shared across many ids often need explicit attention to cache behavior, especially across mixed Reselect versions.
- If the codebase uses React Compiler, treat manual `useCallback` and `useMemo` as suspicious until profiling proves they help.

## NEVER Rules

- NEVER mirror props into state because it feels like caching. Unrelated parent re-renders will eventually blow away local edits or leave your copy stale. Instead choose controlled props, or make the one-shot nature explicit with `initial*` / `default*`.
- NEVER delete an effect dependency because you want `componentDidMount` behavior. That creates stale closures: intervals freeze on old state, fetches use old queries, and subscriptions drift. Instead restructure until the dependency list is honest.
- NEVER chain effects that only set state to trigger other effects because it looks declarative. React documents worst cases with three unnecessary re-renders, and the chain becomes rigid when you add history, replay, or alternate transitions. Instead compute the next state in the event or reducer that knows the full transition.
- NEVER keep duplicated entity objects in state because it makes rendering convenient. Edits will update one copy and not the other, so selection labels, forms, and details panes drift apart. Instead store ids and derive the entity from the canonical collection.
- NEVER reach for `memo` first because it offers a quick local win. You preserve a bad state boundary, add comparison overhead, and make future changes harder to reason about. Instead move volatile state away from expensive subtrees and memoize only the expensive survivors.
- NEVER dispatch Redux actions for every local keystroke just to preserve a "single source of truth". You pay global churn for data no other consumer needs, and typing often gets harder to smooth. Instead buffer locally and dispatch when other consumers or persistence actually need the value.
- NEVER let Redux reducers blindly accept payload shape with `return action.payload` or broad spreads unless the state is a short-lived edit buffer. It is seductive boilerplate removal, but it gives up reducer ownership of invariants and corrupts slices when payloads drift. Instead have reducers project payloads into their own shape.
- NEVER put non-serializable values into Redux state because it seems convenient to keep everything together. You break DevTools, time-travel, replayability, and sometimes UI update assumptions. Instead keep imperatives in middleware, refs, or module scope, and keep store data serializable.

## Edge Cases and Fallbacks

- Interval or subscription keeps restarting: first try honest deps plus updater form; if the process itself must stay alive, use a ref-backed callback or reducer dispatch pattern.
- Prop change should reset a whole form: use a keyed child boundary so React resets the entire subtree automatically.
- Prop change should reset one field only: derive it from ids or adjust during render; avoid effect-based reset because it renders stale UI first.
- Custom hook wraps an effect: keep the API purpose-specific (`useInterval`, `useData`), not lifecycle-shaped. If the hook is effect-like, register it in `eslint-plugin-react-hooks` using `settings.react-hooks.additionalEffectHooks` so dependency linting still applies; this shared setting exists in 6.1.1 and later.
- Selector causes rerender after every action: stop returning fresh arrays and objects from `useSelector`; memoize derived values or split selection into smaller pieces.

## Freedom Calibration

- High freedom: choosing component boundaries, reducer vocabulary, and composition structure.
- Low freedom: effect dependencies, Redux serializability, keyed resets, and normalized data ownership.
- If you are improvising in a low-freedom zone, stop and rewrite the boundary instead.
