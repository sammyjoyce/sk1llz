# Dynamic Color Cookbook (M3 / MCU)⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌‌‌‌​‌‌‌‍‌‌​​​​​​‍‌​​​​‌​​‍​​​‌‌‌‌‌‍​​​​‌​‌​‍​‌‌‌​​‌​⁠‍⁠

Load this file only when: building a custom `DynamicScheme`, extracting a palette from images/logos, debugging desaturated brand colors, or wiring `harmonize()` for non-scheme indicators.

## The seven scheme variants — what each is for

| Variant | What it does to chroma/hue | Use when |
|---|---|---|
| `TonalSpot` (default) | Conservative, balanced, low-chroma | Generic apps, neutral brand, you want "Material default look" |
| `Vibrant` | High chroma on primary, similar hue | Brand-driven apps that want punchy color but Material structure |
| `Expressive` | Hue shifts across roles, rich palette | Playful, lifestyle, creative tools — *the* M3E baseline |
| `Fidelity` | Preserves *source tone* of seed | Brand color **must** look like the seed (#E91E63 stays pink-pink) |
| `Content` | Like Fidelity but tuned for image-derived themes | Photo apps, music players where seed comes from album art |
| `Monochrome` | Single hue, neutral grays | High-contrast, accessibility-first, e-readers |
| `Neutral` | Almost no chroma | Reference docs, enterprise dashboards |
| `Rainbow` | Spread hues across roles | Onboarding flows, kids apps |
| `FruitSalad` | Saturated complementary scheme | Marketing landing pages, single-purpose hero screens |

The novice mistake: shipping `TonalSpot` for a `Vibrant`/`Fidelity` brand and then asking why "Material made my pink boring."

## Selective dynamic theming (the brand-safe pattern)

```kotlin
@Composable
fun BrandTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    dynamicColor: Boolean = true,
    content: @Composable () -> Unit
) {
    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            val ctx = LocalContext.current
            val base = if (darkTheme) dynamicDarkColorScheme(ctx)
                       else dynamicLightColorScheme(ctx)
            // Keep the user's wallpaper-derived surfaces, but lock brand identity
            base.copy(
                primary = Brand.Primary,
                onPrimary = Brand.OnPrimary,
                primaryContainer = Brand.PrimaryContainer,
                onPrimaryContainer = Brand.OnPrimaryContainer,
                error = Brand.Error,
                onError = Brand.OnError,
            )
        }
        darkTheme -> Brand.DarkBaseline   // <-- never omit; pre-API-31 fallback
        else      -> Brand.LightBaseline
    }
    MaterialTheme(colorScheme = colorScheme, content = content)
}
```

Key gotchas:
- Always provide `Brand.LightBaseline` / `Brand.DarkBaseline`. `dynamicLightColorScheme` is not available below API 31. Without a fallback your app **crashes**, not gracefully degrades.
- `.copy()` after `dynamicLightColorScheme(ctx)` is the supported pattern, not extending `lightColorScheme` and patching dynamic values in.
- Never `.copy()` over `surface*` roles in a dynamic scheme — that's exactly the personalization users keep.

## Custom `DynamicScheme` from an image

```kotlin
import com.google.android.material.color.utilities.*

fun schemeFromImage(bitmap: Bitmap, isDark: Boolean): DynamicScheme {
    // 1. Quantize to find the dominant ARGB. Note: this happens in sRGB.
    //    Wide-gamut (P3/Rec2020) source pixels collapse to sRGB nearest.
    val pixels = IntArray(bitmap.width * bitmap.height)
    bitmap.getPixels(pixels, 0, bitmap.width, 0, 0, bitmap.width, bitmap.height)
    val ranked = QuantizerCelebi.quantize(pixels.toList(), 128)
    val seedArgb = Score.score(ranked).first()

    // 2. Pick a variant. SchemeContent for image-derived (preserves source tone).
    val sourceHct = Hct.fromInt(seedArgb)
    return SchemeContent(sourceHct, isDark, /* contrastLevel = */ 0.0)
    // contrastLevel: 0.0 default | 0.5 medium | 1.0 high contrast
    // Use 0.5/1.0 to honor the user's a11y contrast preference, not aesthetics.
}
```

## `harmonize()` for non-scheme colors (status, badges, brand secondaries)

```kotlin
import com.google.android.material.color.utilities.Blend

// Nudges sourceColor toward the hue of the scheme's primary, preserving meaning.
val statusRedHarmonized = Blend.harmonize(
    /* designColor  = */ 0xFFD32F2F.toInt(),
    /* sourceColor  = */ scheme.primary
)
// Red stays red-ish (semantic preserved); the hue shift removes visual clash.
```

When **not** to harmonize:
- True brand colors (your logo's red, your competitor-recognition color)
- Standardized signal colors required by regulation (medical alarm red, traffic-signal green)
- Anything users will perceive as "the wrong color" if shifted

When to harmonize:
- Custom badge backgrounds (achievements, tags)
- Status indicators that are decorative rather than functional
- Tertiary brand accents that exist to differentiate but not identify

## Color fidelity flag — the desaturation fix

The default Material algorithm maps a seed onto a tonal palette and then *picks tones* (40 light / 80 dark) for `primary`, regardless of the seed's original tone. A vivid `#E91E63` (tone ~50, chroma high) becomes a muted pink at tone 40. With **fidelity** enabled, the algorithm shifts the chosen tone to be closer to the seed.

```kotlin
// MCU: use SchemeFidelity instead of SchemeTonalSpot
val scheme = SchemeFidelity(Hct.fromInt(brandSeed), isDark, 0.0)
```

In Material Theme Builder (Figma): toggle "match color" on the input color.

After enabling fidelity, **re-verify on/role contrast**. Fidelity sometimes pushes tones into ranges where `onPrimary` no longer hits 4.5:1.

## Debugging desaturated/wrong colors

| Symptom | Cause | Fix |
|---|---|---|
| Brand color comes out muted | `TonalSpot` + no fidelity | Switch to `Fidelity`/`Content` or enable fidelity flag |
| Photo-derived theme picks the wrong hue | Quantizer picked a background dominant | Pre-crop the source to the subject; use `Score` ranking |
| P3/Rec2020 logo extracts as a different color | HCT quantizer is sRGB-only | Convert source to sRGB display values *before* extraction |
| `primary` is fine but `primaryContainer` clashes | You overwrote `primary` only | Overwrite the whole `primary*` family together |
| Custom red status indicator visually fights primary | Not harmonized | `Blend.harmonize(red, scheme.primary)` |
| Theme Builder output uses `--md-sys-color-surface-tint` | Legacy compatibility | Strip the token and use `surfaceContainer*` roles instead |

## The CSS escape hatch (web)

```css
:root {
  /* Do NOT include --md-sys-color-surface-tint — deprecated since Mar 2023 */
  --md-sys-color-surface: #FEF7FF;
  --md-sys-color-surface-container-lowest: #FFFFFF;
  --md-sys-color-surface-container-low:    #F7F2FA;
  --md-sys-color-surface-container:        #F3EDF7;  /* default for cards */
  --md-sys-color-surface-container-high:   #ECE6F0;
  --md-sys-color-surface-container-highest:#E6E0E9;
  /* ...primary/secondary/tertiary/error/outline as usual */
}

.card { background: var(--md-sys-color-surface-container); }    /* not surface + tint */
.dialog { background: var(--md-sys-color-surface-container-high); }
```
