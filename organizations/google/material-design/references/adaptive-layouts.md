# Adaptive Layouts (window size classes & canonical layouts)⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​‌​​‌‌‍​​​​‌‌‌​‍‌​‌‌‌‌‌​‍​​​​‌‌‌​‍​​​​‌​​‌‍‌​​​​‌​‌⁠‍⁠

Load this file only when: designing for foldables, tablets, or large screens; implementing list-detail / supporting-pane / feed canonical layouts; or debugging "looks fine on phone, broken on tablet" bugs.

## The five window size classes (memorize these, not pixel widths)

| Class | Width (dp) | Typical device | Default nav |
|---|---|---|---|
| **Compact** | 0–599 | Phone portrait, closed foldable, split screen | `NavigationBar` |
| **Medium** | 600–839 | Phone landscape, tablet portrait, foldable open | `NavigationRail` |
| **Expanded** | 840–1199 | Tablet landscape, small desktop window | `PermanentNavigationDrawer` |
| **Large** | 1200–1599 | Desktop, large tablet landscape | `PermanentNavigationDrawer` |
| **Extra-large** | 1600+ | Desktop maximized, ultra-wide | `PermanentNavigationDrawer` |

There is **also** a height class (Compact/Medium/Expanded by height). A landscape phone is `Width=Medium, Height=Compact` — and the height class is what tells you "this is a phone in landscape, don't put a vertical rail of 7 items in it."

## Get the class — never query raw width

### Compose
```kotlin
val windowSizeClass = currentWindowAdaptiveInfo().windowSizeClass
when (windowSizeClass.windowWidthSizeClass) {
    WindowWidthSizeClass.COMPACT  -> CompactScaffold(...)
    WindowWidthSizeClass.MEDIUM   -> MediumScaffold(...)
    WindowWidthSizeClass.EXPANDED -> ExpandedScaffold(...)
}
```

### Views
```kotlin
val metrics = WindowMetricsCalculator.getOrCreate()
    .computeCurrentWindowMetrics(activity)
val windowSizeClass = WindowSizeClass.compute(
    metrics.bounds.width() / activity.resources.displayMetrics.density,
    metrics.bounds.height() / activity.resources.displayMetrics.density,
)
```

### What you must NOT do
```kotlin
// All of these are broken
if (resources.configuration.screenWidthDp >= 600) { ... }    // ignores split screen
if (context.resources.displayMetrics.widthPixels > X) { ... } // wrong unit
if (isTablet) { ... }                                         // foldables, ChromeOS, DeX
```

The reasons hardcoded widths fail in production:
- **Split-screen** on Android shrinks your window without changing the device.
- **Foldables** change width *while the app is running* (book → flat).
- **Landscape phones** are wider than 600dp but ergonomically still phones.
- **ChromeOS / DeX / Samsung DeX** windows are resizable.
- **Picture-in-picture** drops you to ~108dp wide.

## The three canonical layouts

### 1. List-detail
Use for parent-child pairings: inbox + email, files + folder, conversations + messages, settings + category.

| Class | Visible panes | Notes |
|---|---|---|
| Compact | 1 (list **or** detail) | Back button in detail; full-screen each |
| Medium | 1 (recommended) **or** 2 | Two-pane only if content is dense; otherwise it's cramped |
| Expanded+ | 2 (list + detail) | Selected state lives in list |

The most common bug: switching to two-pane at exactly 600dp because "tablet starts there." Read the actual guidance — single-pane is *recommended* in Medium for most use cases.

When transitioning Expanded → Compact (rotation, fold close):
- If something was selected → show *detail* in single pane.
- If nothing was selected → show *list*.
- If the product supports multi-select / list-mode-without-deeper-nav → show whatever was last interacted with.
- The hard rule: be consistent. If you went forward through list → detail, going back must land on list.

### 2. Supporting pane
Use for primary content + a secondary panel that helps complete a task: editor + properties, document + comments, video + chat.

| Class | Layout |
|---|---|
| Compact | Primary full-screen; supporting pane in a bottom sheet |
| Medium | Primary full-screen; supporting pane in a side sheet (modal) |
| Expanded+ | Side-by-side with the primary getting majority width |

Don't make the supporting pane equal width — it stops being "supporting."

### 3. Feed
Use for browsable, repeating content (cards, photos, articles).

| Class | Columns |
|---|---|
| Compact | 1–2 |
| Medium | 2–3 |
| Expanded | 3–4 |
| Large/XL | 4–6 |

Use `LazyVerticalStaggeredGrid` (Compose) or `StaggeredGridLayoutManager` for variable-height items. Fixed-height grids waste space in feeds.

## Navigation pattern matrix

|  | Compact | Medium | Expanded+ |
|---|---|---|---|
| **Primary nav** | `NavigationBar` (3–5) | `NavigationRail` (3–7) | `PermanentNavigationDrawer` (full labels, sections) |
| **Secondary nav** | Modal `NavigationDrawer` | Modal `NavigationDrawer` | Persistent secondary drawer or grouped sections |
| **Tertiary nav** | `TabRow` (scrollable if >4) | `TabRow` | `TabRow` |
| **Contextual content** | Bottom sheet | Side sheet | Inline supporting pane |

Trap: **`NavigationRail.extended` is not in the M3 spec.** It exists in Flutter as a custom widget but it doesn't match official M3 behavior. At the width where you'd want an "extended rail," switch to a `PermanentNavigationDrawer`.

## Foldable-specific gotchas

- **Hinge / table-top posture:** in book/table-top, treat the half above the hinge as the primary surface (e.g., video) and the half below as controls. Use `WindowInfoTracker` to detect the fold.
- **Don't span content across the hinge** unless the device reports `OcclusionType.NONE`. A button under a hinge is unreachable.
- **State must survive rotation/fold** — `rememberSaveable` for Compose, `onSaveInstanceState` for Views. Two-pane → one-pane transitions lose state if you don't.
- **Test the size sequence** Compact → Medium → Expanded → Compact (rotate, unfold, rotate back). Most layout bugs surface in the *return* trip.

## Performance gotchas at large widths

- **A `LazyColumn` of full-bleed items at 1600dp wastes a screen.** Switch to a grid above Expanded.
- **Image assets need `sw800dp` / `sw1200dp` variants** or you'll upscale `mdpi` icons across a 4K display.
- **Touch targets stay 48dp regardless of screen size** — don't grow them with the window. Cursor users on ChromeOS expect normal-sized targets.
- **Hover states matter on Expanded+** because cursor users exist. Wire `:hover` / `Modifier.hoverable` properly.
