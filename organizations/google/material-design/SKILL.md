---
name: google-material-design
description: Apply modern Material Design 3 and Material 3 Expressive using current spec behavior, not M2 or early-M3 folklore. Use when designing or reviewing Android, Compose, Flutter, or web UIs that rely on Material tokens, dynamic color, adaptive navigation, window size classes, or M3 Expressive motion; when upgrading compose-material3 and visuals change unexpectedly; or when requests mention material design, material 3, material you, m3 expressive, dynamic color, surface container, primaryFixed, navigation suite, wide navigation rail, list-detail scaffold, supporting pane, shared element, tonal elevation, or HCT.
---

# Google Material Design

Most Material failures are not "bad taste." They are contract mistakes:

- **Spec drift**: using M2 or 2021-era advice against post-2023 M3.
- **Library-default drift**: Compose upgraded a component default and your screenshots changed.
- **Role misuse**: treating color roles as arbitrary swatches instead of behavioral tokens.

Before changing anything, ask:

1. **Did the UI change after a dependency bump?** If yes, suspect component defaults before you "fix" the palette.
2. **Is the problem about containment, not depth?** If yes, choose a different `surfaceContainer*` role, not more dp.
3. **Does the accent need to stay visually stable across light/dark or wallpaper changes?** If yes, use fixed roles or selective dynamic theming, not a blanket dynamic scheme.
4. **Is the layout bug caused by resize/posture changes?** If yes, stop writing raw width checks and switch to the adaptive scaffolds.

## What changed that still trips up senior teams

### Surface containers replaced most manual elevation styling

Post-2023 M3 moved components toward explicit surface roles and away from "surface plus tint math." In Compose Material 3, many defaults now encode the role directly:

- `ElevatedCard` and `ElevatedButton` use `SurfaceContainerLow`
- `FilledCard`, `SearchBar`, and `TextField` use `SurfaceContainerHighest`
- `AlertDialog` and `DatePicker` use `SurfaceContainerHigh`
- menus and modal drawers moved to new container roles, and several previous tonal elevations were reduced to `0.dp`

The practical consequence: if a component looks "too lifted" after you customize it, check whether the library already applied a container role before you add your own tonal elevation on top. In current Compose releases, restoring old drawer or menu elevation values is usually reintroducing pre-migration behavior, not fixing a bug.

### Constructors without fixed roles or surface containers are already legacy

Compose deprecated `ColorScheme` constructors that omit fixed roles and the new surface-container roles. If you hand-build a scheme and ignore `primaryFixed`, `secondaryFixed`, `surfaceContainerHigh`, and friends, you are freezing an older spec snapshot into your app.

Use fixed roles when the accent must keep roughly the same perceived tone in light and dark themes. Use normal `primary` roles when you want Material's dynamic behavior.

### Adaptive navigation defaults are more conservative than most teams assume

For Compose adaptive navigation, the default suite is:

- **Navigation bar** when width is compact, height is compact, or the device is in tabletop posture
- **Navigation rail** otherwise

Drawer is not the default "because the screen is wide." If you want a drawer on expanded desktop/tablet layouts, make that an explicit information-architecture decision, not an automatic breakpoint reflex.

### Default window classes are three-tier unless you opt into five

Material 3's current window size APIs return `Compact`, `Medium`, and `Expanded` by default. Large and extra-large width classes exist only when you explicitly opt in. Do not build a five-breakpoint system unless the product truly behaves differently on desktop vs. tablet-large. Most apps only need three.

## Decision rules experts actually use

### Choosing surface roles

- `surfaceContainerLow` means "slightly separated from the base surface"
- `surfaceContainer` means "default contained surface"
- `surfaceContainerHigh/Highest` mean "needs stronger containment"

These are **containment** signals, not a z-axis. Resting elevation should usually stay at levels `0` through `+3`; `+4` and `+5` are interaction states such as hover and dragged, not steady-state cards.

If the background is photographic, mapped, or gradient-heavy, tonal separation will disappear. Keep the container role, then add a small shadow or a 32% scrim. On busy backgrounds, shadow is a contrast tool, not a style choice.

### Choosing dynamic color strategy

- **Brand must stay recognizable**: start from the dynamic scheme, then overwrite the brand-critical family or fixed roles.
- **Photo- or content-derived theme**: use `SchemeContent` or `SchemeFidelity`, not the default `TonalSpot`.
- **Accessibility request**: use `contrastLevel` for user preference only. The range is `-1` to `1`; treating `1.0` as a design flourish flattens hierarchy and makes everything feel equally loud.

Two non-obvious MCU behaviors matter in production:

1. Image quantization and HCT extraction are effectively sRGB-oriented. Wide-gamut source colors can collapse in ways brand teams interpret as "wrong."
2. MCU contains a `DislikeAnalyzer` that lightens dark yellow-greens considered broadly unpleasant. The current heuristic targets hues roughly `90..111`, chroma above `16`, and tone below `65`, then lifts them toward tone `70`. If themes extracted from foliage or food imagery skew away from the source, that may be intentional library behavior, not a bug.

### Choosing navigation and pane scaffolds

- Use `NavigationSuiteScaffold` first. Override the type only when labels add more value than reclaimed content width.
- Use `ListDetailPaneScaffold` and `SupportingPaneScaffold` before inventing your own two-pane rules.
- Keep list-detail single-pane through compact and usually medium. Split views that trigger at `600dp` by habit almost always feel cramped.
- If you truly need a labeled rail, prefer the platform's `WideNavigationRail` over a homemade "extended rail" pattern.

### Choosing motion

In M3 Expressive, spring families are semantically split:

- `spatial*` springs are for position, size, and geometry
- `effects*` springs are for color and opacity

Do not mix them. Overshoot on opacity reads as flicker. This is exactly why Material 3 moved bottom-sheet motion away from a spring in one of the Compose updates: visible overshoot looked broken.

## Anti-patterns

**NEVER treat a post-upgrade visual diff as proof your palette is wrong** because the seductive path is to tweak tokens until screenshots match old builds. The concrete consequence is weeks of fighting upstream defaults when the real change was a library migration to new `surfaceContainer*` roles or fixed-role constructors. Instead diff the component defaults for your `compose-material3` version first.

**NEVER hand-paint `surfaceTint` overlays** because old blog posts, generated CSS, and Stack Overflow snippets make it look like "how tonal elevation works." The non-obvious problem is that modern M3 components already resolve containment through container roles and built-in tonal elevation behavior; adding your own tint creates double-elevation and dark-theme drift. Instead assign the correct `surfaceContainer*` role or use `Surface(tonalElevation = ...)` and let the library resolve it.

**NEVER use a permanent drawer just because the window is wide** because labels feel safer than icons and wide layouts tempt teams to spend width on navigation chrome. The consequence is lost content area, awkward medium-width behavior, and divergence from the adaptive suite's bar-or-rail default. Instead start with `NavigationSuiteScaffold`, then override to drawer only when the IA truly benefits from always-visible labels and sections.

**NEVER split list-detail at `600dp` by reflex** because "tablet starts at 600" is an old Android heuristic that still sounds pragmatic. The consequence is a cramped medium layout where neither pane has enough breathing room and touch targets compete with content. Instead use the adaptive pane scaffolds and keep medium single-pane unless dense scanning is measurably better.

**NEVER crank `contrastLevel` or choose `Fidelity` just to make the UI feel punchier** because the result looks vivid in isolated mocks. The consequence is flattened hierarchy, role pairs that need re-verification, and a theme that stops behaving like Material's accessibility model. Instead use fidelity for source preservation and contrast for user need.

**NEVER shrink hit targets to "undo" minimum interactive sizing** because large fonts, desktop windows, and icon-only actions make the padded version look visually off. The consequence is misaligned visuals and unreliable tap regions. Instead keep the 48dp hit area and align with `MinimumInteractiveTopAlignmentLine` and `MinimumInteractiveLeftAlignmentLine` when the visual edge must line up cleanly.

## Recovery playbook

- **"Dark theme looks flat."** Check whether you are stacking the same surface role inside itself. Change the inner role before adding more elevation.
- **"The brand color became muddy."** Verify the scheme variant first. `TonalSpot` is the usual culprit; switch to `Fidelity` or selective dynamic theming.
- **"Large-screen nav feels wrong."** Re-test with the default adaptive suite. If bar/rail suddenly feels fine, your custom drawer rule was the bug.
- **"Wallpaper extraction picked a gross olive."** Confirm whether `DislikeAnalyzer` or source-image dominance is affecting the seed before overriding the whole palette.
- **"Buttons look misaligned after enforcing 48dp."** Use `MinimumInteractiveTopAlignmentLine` / `MinimumInteractiveLeftAlignmentLine`; do not remove the padding.

## Load deeper references only when needed

- Before building or debugging custom dynamic schemes, READ `references/dynamic-color-cookbook.md`
- Before implementing shared elements, container transforms, or motion choreography, READ `references/motion-recipes.md`
- Before exporting or auditing tokens, READ `references/token-tables.md`
- Before designing list-detail, supporting-pane, foldable, or desktop layouts, READ `references/adaptive-layouts.md`

Do NOT load the reference files for routine "make this more Material" work. This file should be enough for most review, migration, and component-level decisions.

## Verify against current sources when stakes are high

- `developer.android.com/jetpack/androidx/releases/compose-material3`
- `developer.android.com/develop/ui/compose/layouts/adaptive/build-adaptive-navigation`
- `developer.android.com/codelabs/build-adaptive-apps`
- `github.com/material-foundation/material-color-utilities`
