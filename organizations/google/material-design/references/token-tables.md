# M3 Token Tables (current spec)⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌​‌​‌​‌‍​‌​‌‌​‌​‍‌‌​‌​​‌‌‍‌​‌‌‌​‌‌‍​​​​‌​‌​‍‌‌​‌​‌​‌⁠‍⁠

Load this file only when: implementing the full token system in CSS/Compose, building a design-token export, or auditing an existing token file for spec drift.

## Color roles — full list (post-2023)

```css
:root {
  /* Primary family */
  --md-sys-color-primary:                 #6750A4;
  --md-sys-color-on-primary:              #FFFFFF;
  --md-sys-color-primary-container:       #EADDFF;
  --md-sys-color-on-primary-container:    #21005D;

  /* Secondary family */
  --md-sys-color-secondary:               #625B71;
  --md-sys-color-on-secondary:            #FFFFFF;
  --md-sys-color-secondary-container:     #E8DEF8;
  --md-sys-color-on-secondary-container:  #1D192B;

  /* Tertiary family */
  --md-sys-color-tertiary:                #7D5260;
  --md-sys-color-on-tertiary:             #FFFFFF;
  --md-sys-color-tertiary-container:      #FFD8E4;
  --md-sys-color-on-tertiary-container:   #31111D;

  /* Error family */
  --md-sys-color-error:                   #B3261E;
  --md-sys-color-on-error:                #FFFFFF;
  --md-sys-color-error-container:         #F9DEDC;
  --md-sys-color-on-error-container:      #410E0B;

  /* Surface — note tone shift in 2023 (was 99, now 98) */
  --md-sys-color-surface:                 #FEF7FF;  /* tone 98 in light */
  --md-sys-color-on-surface:              #1C1B1F;
  --md-sys-color-on-surface-variant:      #49454F;

  /* Surface containers — REPLACED elevation tints */
  --md-sys-color-surface-container-lowest:#FFFFFF;
  --md-sys-color-surface-container-low:   #F7F2FA;
  --md-sys-color-surface-container:       #F3EDF7;  /* default for cards */
  --md-sys-color-surface-container-high:  #ECE6F0;
  --md-sys-color-surface-container-highest:#E6E0E9;
  --md-sys-color-surface-dim:             #DED8E1;
  --md-sys-color-surface-bright:          #FEF7FF;

  /* Outline & inverse */
  --md-sys-color-outline:                 #79747E;
  --md-sys-color-outline-variant:         #CAC4D0;
  --md-sys-color-inverse-surface:         #313033;
  --md-sys-color-inverse-on-surface:      #F4EFF4;
  --md-sys-color-inverse-primary:         #D0BCFF;

  /* Scrim */
  --md-sys-color-scrim:                   #000000;  /* used at 32% opacity */

  /* DEPRECATED — do not include in new themes:
     --md-sys-color-surface-tint
     --md-sys-color-surface-variant   (now == surface-container-highest)
  */
}
```

Dark theme **darkens** surface containers further than M2 (post-2023 update). Neutral palette chroma was **increased from 4 to 6** in the same update — if your tokens still use chroma 4, regenerate.

## Type scale (M3 + M3E emphasized variants)

| Role | Size | Weight | Use |
|---|---|---|---|
| display-large | 57sp | 400 | Hero headlines |
| display-medium | 45sp | 400 | Large display |
| display-small | 36sp | 400 | Smaller display |
| headline-large | 32sp | 400 | High-emphasis headers |
| headline-medium | 28sp | 400 | Section headers |
| headline-small | 24sp | 400 | Sub-headers |
| title-large | 22sp | 400 | Card titles |
| title-medium | 16sp | 500 | Tab/list titles |
| title-small | 14sp | 500 | Compact titles |
| body-large | 16sp | 400 | Primary body |
| body-medium | 14sp | 400 | Secondary body |
| body-small | 12sp | 400 | Captions |
| label-large | 14sp | 500 | Buttons, prominent labels |
| label-medium | 12sp | 500 | Nav labels |
| label-small | 11sp | 500 | Timestamps |

M3E adds **emphasized** variants of each role (variable-font weight axis) for hero moments. Don't apply emphasized type globally — it loses its emphasis function.

## Shape scale

```css
:root {
  --md-sys-shape-corner-none:        0px;
  --md-sys-shape-corner-extra-small: 4px;
  --md-sys-shape-corner-small:       8px;
  --md-sys-shape-corner-medium:      12px;
  --md-sys-shape-corner-large:       16px;
  --md-sys-shape-corner-extra-large: 28px;
  --md-sys-shape-corner-full:        9999px;  /* pill */
}

/* Default component → shape mapping */
.chip   { border-radius: var(--md-sys-shape-corner-small); }
.card   { border-radius: var(--md-sys-shape-corner-medium); }
.dialog { border-radius: var(--md-sys-shape-corner-extra-large); }
.fab    { border-radius: var(--md-sys-shape-corner-large); }
.button { border-radius: var(--md-sys-shape-corner-full); }
```

M3E adds a **35-shape library** (squircles, polygons, scallops) and a *shape-morph animation* spec for transitioning between shapes. Use these for decorative elements (avatars, image crops, hero containers) — not for buttons, where the pill shape carries semantic meaning.

## Elevation levels

| Level | dp | Resting? | Examples |
|---|---|---|---|
| 0 | 0 | yes | Background, disabled, flat surfaces |
| +1 | 1 | yes | Resting search bar, list-detail divider |
| +2 | 3 | yes | Resting card, snackbar, bottom app bar |
| +3 | 6 | yes | FAB resting, top app bar (scrolled) |
| +4 | 8 | **NO** | Reserved: hover state |
| +5 | 12 | **NO** | Reserved: dragged state |

M2's "elevation 16dp nav drawer / 24dp modal" values are **gone** in M3. Use surface container roles for emphasis instead of cranking dp.

## State layer opacities (exact)

| State | Opacity |
|---|---|
| Hover | **0.08** |
| Focus | **0.12** |
| Pressed | **0.12** |
| Dragged | **0.16** |
| Disabled (element) | **0.38** *(not a layer — actual element opacity)* |
| Disabled (container) | **0.12** *(actual container opacity)* |
| Scrim | **0.32** *(scrim color role)* |

## Motion duration tokens

| Token | ms | Use |
|---|---|---|
| short1 | 50 | Micro feedback (state layer) |
| short2 | 100 | Toggle switch |
| short3 | 150 | Checkbox flick |
| short4 | 200 | Small expand/collapse, fade |
| medium1 | 250 | Shared axis, fade through |
| medium2 | 300 | Container transform, card expand |
| medium3 | 350 | Complex chained animation |
| medium4 | 400 | Page transition (handheld upper bound) |
| long1 | 450 | Large area transitions (large screens) |
| long2 | 500 | Choreographed multi-element |
| long3 | 550 | Hero moment |
| long4 | 600 | Maximum — use only for hero |

Above 600ms, users perceive a transition as broken, not premium.

## Compose Material3 mapping

```kotlin
MaterialTheme.colorScheme.surfaceContainer        // not .surface + tint
MaterialTheme.colorScheme.surfaceContainerLow
MaterialTheme.colorScheme.surfaceContainerHigh
MaterialTheme.colorScheme.surfaceContainerHighest
MaterialTheme.colorScheme.surfaceContainerLowest

MaterialTheme.shapes.medium                       // ShapeDefaults.Medium = 12.dp
MaterialTheme.typography.bodyLarge                // 16sp / 400
```

`Surface(tonalElevation = X.dp)` is the *correct* M3 way to apply tonal elevation in Compose — it does the surface-tint blending under the hood and respects dynamic color. Don't `Modifier.background(surfaceTint.copy(alpha = elevationAlpha))` by hand.
