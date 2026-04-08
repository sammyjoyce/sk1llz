---
name: google-material-design
description: Apply Material Design 3 / Material 3 Expressive (M3E) the way Google's design team actually uses it in 2025 — surface-container roles instead of elevation tints, tonal vs. shadow elevation tradeoffs, HCT/dynamic-color pitfalls, motion-physics springs, canonical adaptive layouts, and the M3E "containment + emphasis" tactics that beat flat design 4× in eye-tracking. Use when building or reviewing Android, Flutter, Compose, or web UIs that target M3/M3E; when migrating from M2 or pre-2023 M3; when wiring dynamic color, theme builders, or wallpaper extraction; when picking navigation patterns across window size classes; when an interface "feels Material but looks wrong"; or when keywords like material design, material 3, material you, m3 expressive, dynamic color, tonal palette, surface tint, surface container, navigation rail, FAB, canonical layout, shared element transition, container transform, or jetpack compose theming appear.
---

# Google Material Design (M3 / M3 Expressive)

Material is no longer "flat with shadows." Two structural shifts since 2023 invalidate most pre-2024 tutorials and most things you "remember" about Material:

1. **Tone-based surfaces (Mar 2023)** replaced elevation-tied `surface-tint` overlays. `surfaceTint` is **deprecated**. Use the five `surfaceContainer*` roles.
2. **Material 3 Expressive (May 2025)** reverses flat-design minimalism with containment, stronger color, emphasized type, and a spring-physics motion system. Eye-tracking shows users find primary actions **4× faster** and the elderly/young performance gap collapses. M3E is additive — *not* M4, *not* a deprecation of M3.

If a tutorial still says "increase elevation to tint a card" or "use surfaceTint with elevationOverlay," it predates the 2023 update. Reject it.

## Before you touch the design system, ask

1. **Is this a brand-first product (banking, medical, brand-led commerce)?** → Disable wallpaper-derived dynamic color for `primary`/`onPrimary`. Use *selective dynamic theming*: dynamic surfaces, static brand accents. (Pattern: copy a `dynamicLightColorScheme` and overwrite `primary`/`onPrimary` only.)
2. **Is the UI sitting on photos, gradients, or busy backgrounds?** → Tonal elevation alone will disappear. Add a *small* `Modifier.shadow` or a 32%-opacity scrim. Default M3 tonal is calibrated for solid surfaces.
3. **Am I targeting Android < 12?** → `dynamicLightColorScheme` returns null below API 31. You **must** ship a baseline `lightColorScheme`/`darkColorScheme` fallback or your app will crash on >50% of in-market devices.
4. **Am I about to write a `width < 600` media query?** → Stop. Use the five window size classes (Compact 0–599, Medium 600–839, Expanded 840–1199, Large 1200–1599, Extra-Large 1600+). They handle split-screen, foldables, and landscape phones — bare width queries don't.
5. **Is this an M3E component (FAB menu, button group, split button, toolbar, loading indicator)?** → As of late 2025 these are **alpha** in `androidx.compose.material3`. Don't ship them in production-critical paths without a fallback.

## Decisions experts get right and beginners get wrong

### Tonal vs. shadow elevation (the choice that ruins dark mode)

| Situation | Use | Why |
|---|---|---|
| Default Material card on a solid surface | **Tonal only** | Adapts to light/dark and dynamic color automatically |
| Card over a photo, gradient, video, or map | **Tonal + small shadow** | Shadows give the only reliable separation against busy backgrounds |
| Anything in dark theme that needs to "lift" | **Tonal** | Shadows on dark backgrounds vanish into the noise; tonal *brightens* the surface as it rises |
| Dialogs, menus, FAB at rest | **Tonal + shadow** | These need unambiguous separation in both themes |
| "It looks flat in dark mode" | You're using shadow only | Switch to tonal (M3 default), shadows are a *selective* tool now |

### `surfaceContainer*` is not a synonym for elevation

The five surface container roles are **not** ordered by z-distance. They are ordered by *emphasis against the base surface*. Use them for containment, not for "this card is higher than that one." Migration:

| Pre-2023 M3 | After Mar 2023 |
|---|---|
| `surface` + tint at elev +1 | `surfaceContainerLow` |
| `surface` + tint at elev +2 | `surfaceContainer` *(default for cards)* |
| `surface` + tint at elev +3 | `surfaceContainerHigh` |
| `surface` + tint at elev +4 / +5 | `surfaceContainerHighest` *(elev +4/+5 deprecated as resting states)* |
| `surfaceVariant` | `surfaceContainerHighest` |

M3 elevation is now **6 levels (0 → +5)**, but only **0 → +3 are valid resting states**. Levels +4 and +5 are reserved for hover and dragged states. If a designer asks for "elev +5 cards," they're using M2 mental models.

### Dynamic color: the three things the docs bury

1. **HCT is sRGB-only.** The Material Color Utilities quantizer converts images through an sRGB canvas context. Wide-gamut (P3, Rec2020) source pixels collapse to their sRGB nearest. If you're extracting from HDR/P3 photography, expect color shifts. Don't use HCT/MCU as a general color library.
2. **Default extraction desaturates your brand.** The color algorithm maps any input to a tonal palette and then picks tone 40 (light) / 80 (dark) for `primary`. A vibrant `#E91E63` becomes a muted desaturated pink. **Enable `fidelity`** (or use `SchemeFidelity`/`SchemeContent` variants) to keep tones close to the source. The price: slightly less consistent contrast — re-verify on/role pairings.
3. **`harmonize()` is for non-scheme colors only.** Use it to nudge status colors (custom red/green/yellow indicators) toward your `primary` hue so they don't visually clash, *without* losing semantic meaning (red stays red-ish). **Never harmonize true brand colors** — that's exactly what they should never do.

### Dynamic scheme variants — pick by intent, not vibe

`TonalSpot` (default, conservative) · `Vibrant` (more chroma, brand-friendly) · `Expressive` (variable hues, playful) · `Fidelity` / `Content` (preserve source tone) · `Monochrome` · `Neutral` · `Rainbow` · `FruitSalad`. The novice mistake is shipping `TonalSpot` for a brand that needs `Vibrant`/`Fidelity`, then complaining "Material made my colors boring."

### The medium-width (600–839dp) trap

The official guidance is *recommended single-pane* in medium, not two-pane. Most teams reflexively split panes at 600dp because that's where "tablets" begin in their head. At 600–839dp the panes are too cramped, both panes lose breathing room, and touch targets crowd. Use two-pane in medium **only** for information-dense list-detail with quick scanning (mail, files). For everything else, single-pane until 840dp.

### Navigation pattern by window size — the right table

| Width | Primary nav | Note |
|---|---|---|
| 0–599 (Compact) | Bottom `NavigationBar` (3–5 items) | Never put a drawer here; rail is too wide |
| 600–839 (Medium) | `NavigationRail` (3–7 items) | "Extended rail" is **not in the M3 spec** — that's a Flutter custom and a known anti-pattern |
| 840+ (Expanded/Large/XL) | `PermanentNavigationDrawer` (full labels) | Switch to drawer; rail loses information value |

## When implementing, use these exact numbers

- **State layer opacities (M3 spec, not your taste):** hover **0.08**, focus **0.12**, pressed **0.12**, dragged **0.16**. Disabled is **not** a state layer — set element opacity to **0.38** and container/surface to **0.12**.
- **Scrim:** color role `scrim` at **32%** opacity. Not 50%, not "rgba(0,0,0,0.5)".
- **Touch target:** **48dp minimum** on Android. iOS HIG is 44pt — they are not interchangeable; if your team copies iOS specs you'll fail Material a11y review.
- **Animation budget on handhelds:** stay in **200–400ms** for transitions; never exceed 500ms unless choreographing a hero moment. Long-running transitions feel broken, not premium.
- **Spring physics tokens (M3E motion):** `spatial*` springs animate position/size; `effects*` springs animate color/opacity. **Never mix** — using a spatial spring on opacity produces visible overshoot artifacts.

## Anti-patterns

**NEVER use `surfaceTint` with elevation overlays.** It's seductive because Material Theme Builder *still* emits the `--md-sys-color-surface-tint` token for legacy compatibility, and old Stack Overflow answers tell you to multiply it by elevation. **Consequence:** double-tinting against the new `surfaceContainer*` roles, wrong contrast in dark mode, and a UI that subtly drifts off-spec on every component. **Instead:** delete `surfaceTint` from custom themes and assign components to `surfaceContainerLow/Container/High/Highest` based on *containment emphasis*, not depth.

**NEVER let a surface "pop in" or fade in from nothing.** It looks fine in isolation and reads as "snappy." **Consequence:** breaks the material metaphor — physical paper cannot teleport — and users perceive the screen as glitching, not as fast. Eye-tracking studies attribute click hesitation to this. **Instead:** surfaces enter via *container transform* (an existing element expands into the new surface), *shared axis* (slide for sibling navigation), or *fade through* (only for unrelated content swaps with no spatial relationship).

**NEVER blindly enable `dynamicColor = true` for a brand product.** Personalization sounds great. **Consequence:** your bank app's logo color disappears, error states clash with someone's lavender wallpaper, and Compliance asks why the "send money" button is now mauve. **Instead:** enable dynamic color for `surface*`/`background` roles only, and overwrite `primary`/`onPrimary`/`error` from your brand baseline (`dynamicLightColorScheme(ctx).copy(primary = Brand.Primary, ...)`).

**NEVER design buttons with whitespace as the only signifier.** Flat-design dogma says "remove what's not needed." **Consequence:** NN/g, Burmistrov, Lücken, *and* Google's own M3E research all show measurable click hesitation, hesitancy time roughly doubling, and a stark age-related performance gap. M3E redesigns improved *send-button discovery* 4× by adding container, contrast, and color. **Instead:** apply M3E **containment** (rounded backgrounds, common-region grouping, contrasted fills) on every primary action. Whitespace alone is not a signifier.

**NEVER use shadow elevation as the primary depth cue in dark theme.** It's the M2 reflex. **Consequence:** shadows blend into dark backgrounds and the hierarchy collapses. **Instead:** rely on *tonal* elevation (which brightens the surface) and add shadow only where you also need separation against a busy/photographic background.

**NEVER hardcode `if (width < 600)` style breakpoints.** They feel pragmatic. **Consequence:** breaks split-screen multitasking, breaks foldables in book posture, treats landscape phones as "tablets," and produces the exact hardcoded-magic-number bugs adaptive layouts exist to prevent. **Instead:** use `WindowSizeClass.calculateFromSize(...)` (Compose) / `currentWindowAdaptiveInfo()` and switch on the official five classes.

**NEVER use Material Theme Builder's generated CSS verbatim.** It still emits `--md-sys-color-surface-tint` and pre-deprecation overlay variables. **Consequence:** drift from current spec. **Instead:** generate, then strip `surface-tint*` and add the `surfaceContainer{Lowest,Low,,High,Highest}` roles by hand.

**NEVER ship M3E alpha components on a critical user path.** They look slick in demos. **Consequence:** alpha API surface, breaking changes between releases, missing accessibility wiring. **Instead:** use the stable `androidx.compose.material3` components and reserve M3E (button groups, FAB menu, split buttons, toolbars, loading indicator, motion-physics springs) for hero moments behind a feature flag.

**NEVER ship more than 1–2 hero moments per product.** Every "delight" feels like a win. **Consequence:** when everything is a hero, nothing is — emphasis becomes noise and the screen reads as visually loud. **Instead:** identify the single emotionally pivotal interaction (compose-send, complete-purchase, capture, share) and concentrate shape/motion/color emphasis there.

## Decision trees and failure recovery

- **"My dark theme card is invisible."** → You're using shadow elevation. Switch to tonal: `Surface(tonalElevation = 3.dp)` (Compose) or `surfaceContainerHigh` token. If still invisible, you're nesting same-role surfaces — give the inner surface a *different* container role, not a higher elevation.
- **"Dynamic colors look muddy / desaturated."** → You're on `TonalSpot` with default fidelity. Switch scheme variant to `Vibrant` or enable color fidelity. If you control the source (logo upload), enrich the input chroma before extracting.
- **"Container transforms look janky in RecyclerView/list-to-detail."** → Build `transitionName` *dynamically per item* (use the domain id, not XML), call `postponeEnterTransition()` from the **parent** fragment when nested, disable Glide's image transformations on the shared image, and call `startPostponedEnterTransition()` from `doOnPreDraw` on both success and error. (Full code: see `references/shared-element-transitions.md`.)
- **"Two-pane layout feels cramped on a tablet in portrait."** → Tablet portrait is usually Medium (600–839dp), where single-pane is *recommended*. Reserve two-pane for Expanded+ (840dp+) or for genuinely list-detail mail/files use cases.
- **"My app passes a11y on Pixel and fails on Samsung."** → Touch targets are 48dp on Android *regardless* of OEM. Check that the visual icon (24dp) is centered inside a 48dp tap container. Don't shrink the container to fit the icon.

## Loading more context

Most of the time this file is enough. Load deeper references **only** when the task matches:

- **Building a custom dynamic scheme, mapping logos to schemes, or debugging desaturated colors** → READ `references/dynamic-color-cookbook.md`
- **Wiring container transforms, shared element transitions, or fade-through across fragments/screens** → READ `references/motion-recipes.md`
- **Implementing the full M3 token table (color, type, shape, elevation, motion springs) in CSS or Compose** → READ `references/token-tables.md`
- **Designing for foldables, large screens, or canonical layouts (list-detail, supporting pane, feed)** → READ `references/adaptive-layouts.md`

Do NOT load these for general "make this look Material" tasks — the SKILL.md alone covers 90% of cases and the deeper files are dense.

## Authoritative sources (for verification, not skim-reading)

- m3.material.io/styles/color (current scheme & roles)
- m3.material.io/blog/tone-based-surface-color-m3 (the 2023 deprecation)
- m3.material.io/blog/building-with-m3-expressive (the 2025 M3E shift)
- m3.material.io/foundations/layout/canonical-layouts (window size classes & layouts)
- github.com/material-foundation/material-color-utilities (HCT, MCU, scheme variants)
