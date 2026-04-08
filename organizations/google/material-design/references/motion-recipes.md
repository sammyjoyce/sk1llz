# Motion Recipes (M3 + M3 Expressive)⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​​​‌​​​‍​‌​​‌‌‌‌‍​‌‌‌​​​​‍‌​‌​​​‌​‍​​​​‌​‌​‍​​​​​​​‌⁠‍⁠

Load this file only when: implementing container transform, shared element transitions, fade-through, or wiring M3E motion-physics springs.

## The four canonical M3 transition patterns

| Pattern | Use when | Duration | Token |
|---|---|---|---|
| **Container transform** | An element grows into a new screen (card → detail) | 300ms | `motion-duration-medium2` |
| **Shared axis (X/Y/Z)** | Sibling navigation, tabs, stepped flows | 250ms | `motion-duration-medium1` |
| **Fade through** | Switching unrelated content (bottom-nav destinations) | 250ms | `motion-duration-medium1` |
| **Fade** | Anything appearing/disappearing in place (chips, snackbars) | 200ms | `motion-duration-short4` |

The constraint: **a surface never just appears.** If none of the above fit, your information architecture is wrong, not the motion system.

## Easing curves (M3 standard, pre-spring)

```css
:root {
  --md-sys-motion-easing-emphasized:           cubic-bezier(0.2, 0.0, 0, 1.0);
  --md-sys-motion-easing-emphasized-decelerate:cubic-bezier(0.05, 0.7, 0.1, 1.0);
  --md-sys-motion-easing-emphasized-accelerate:cubic-bezier(0.3, 0.0, 0.8, 0.15);
  --md-sys-motion-easing-standard:             cubic-bezier(0.2, 0.0, 0, 1.0);
  --md-sys-motion-easing-standard-decelerate:  cubic-bezier(0, 0, 0, 1);
  --md-sys-motion-easing-standard-accelerate:  cubic-bezier(0.3, 0, 1, 1);
}
```

Use **emphasized** for prominent elements (page transitions, FABs, dialogs) and **standard** for everything else. Never use linear easing on a position change — physical objects don't move at constant velocity.

## M3E motion-physics springs (May 2025)

M3E replaces fixed easing curves with **physical springs**. Two families that **must not be mixed**:

- `spatial*` springs — for *position, size, rotation*. Allow visible overshoot/settle.
- `effects*` springs — for *color, opacity, alpha*. No overshoot (overshoot on opacity = visible flicker).

```kotlin
import androidx.compose.material3.MaterialTheme
import androidx.compose.animation.core.spring

// Compose alpha API (subject to change)
val spatial = MaterialTheme.motionScheme.defaultSpatialSpec<Float>()
val effects = MaterialTheme.motionScheme.defaultEffectsSpec<Float>()

// CORRECT: spatial for size, effects for color
Modifier
  .animateContentSize(animationSpec = spatial)
  .background(animateColorAsState(target, animationSpec = effects).value)

// WRONG: spatial spring on opacity → visible bounce on fade
animateFloatAsState(targetAlpha, animationSpec = spatial)  // do not do
```

The motion scheme presets (`MotionScheme.standard()`, `.expressive()`) tune *both* families together. Pick a scheme, don't hand-tune individual springs unless you're designing a hero moment.

## Container transform (Compose)

```kotlin
// SharedTransitionScope-based container transform (Compose 1.7+)
SharedTransitionLayout {
    AnimatedContent(targetState = selectedItem) { item ->
        if (item == null) {
            ListView(onSelect = { selectedItem = it })
        } else {
            DetailView(
                item = item,
                modifier = Modifier.sharedBounds(
                    sharedContentState = rememberSharedContentState(key = "card-${item.id}"),
                    animatedVisibilityScope = this@AnimatedContent,
                    boundsTransform = { _, _ ->
                        spring(stiffness = Spring.StiffnessMediumLow)
                    },
                ),
            )
        }
    }
}
```

Key rules:
- The shared key (`"card-${item.id}"`) **must** be stable across composables. Don't use `item.position`.
- Both source and destination must be in the same `SharedTransitionLayout`.
- Use `sharedBounds` for shape morphs, `sharedElement` for identical content.

## Shared element transitions (View system, Fragments)

The four hidden mistakes that cost senior Android engineers days:

### 1. Static `transitionName` in `RecyclerView`
```kotlin
// WRONG: every item has the same name
android:transitionName="@string/transition_image"

// CORRECT: build the name dynamically per item, in BOTH source and destination
val name = context.getString(R.string.transition_image, item.id)
ViewCompat.setTransitionName(imageView, name)
```

### 2. `postponeEnterTransition()` from a *child* fragment when nested
```kotlin
// WRONG: child fragment postpones, parent already drew
override fun onCreateView(...) {
    postponeEnterTransition()  // never resumes
    ...
}

// CORRECT: postpone from the parent fragment that owns the FragmentManager
parentFragment?.postponeEnterTransition()
parentFragment?.startPostponedEnterTransition()
```

### 3. Glide image transformations break the matrix
```kotlin
// WRONG: Glide resizes/crops, the source image arrives with a different matrix
// than the destination expects, causing a visible "snap" mid-transition.

// CORRECT: disable transformations on the shared image
Glide.with(view)
    .load(url)
    .dontTransform()                       // critical
    .dontAnimate()
    .listener(transitionReadyListener)
    .into(imageView)
```

### 4. Calling `startPostponedEnterTransition()` only on success
```kotlin
// WRONG: error path leaves the transition postponed → frozen UI
val listener = object : RequestListener<Drawable> {
    override fun onResourceReady(...): Boolean {
        startPostponedEnterTransition()
        return false
    }
    // missing onLoadFailed override → app freezes on image error
}

// CORRECT: resume in BOTH success and error
override fun onLoadFailed(...): Boolean {
    startPostponedEnterTransition()        // critical
    return false
}
```

For non-image transitions, use `view.doOnPreDraw { startPostponedEnterTransition() }` on the destination's root.

### Bonus: `RecyclerView` returning to a different position

Save the source item id, compare on return, and start the transition only when the matched item is laid out (use a `OnLayoutChangeListener` on the `RecyclerView` itself).

## Choreography rules of thumb

- Handheld: stay in **200–400ms**. Above 500ms feels broken.
- Container transform: ~**300ms** (medium2). Faster feels jarring; slower feels syrupy.
- Shared axis: **250ms** (medium1).
- Hero moment: up to ~**500ms** (long2) — use sparingly.
- Choreograph **stagger** by 16–33ms between elements (one or two frames). Larger stagger reads as broken.
- Decelerate (`emphasized-decelerate`) for *enter*; accelerate for *exit*. Mixing them feels uncanny.
