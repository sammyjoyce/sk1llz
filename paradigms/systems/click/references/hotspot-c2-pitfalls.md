# HotSpot C2 — Field Notes & Failure Modes

Load this file ONLY when diagnosing a specific C2 performance problem, tuning
inlining/EA, or writing code that must stay on the C2 fast path. Do NOT load
for generic "how does a JIT work" explanations.

---

## The Inlining Bible (memorize these numbers)

C2 inlining is a chain. Break any link, every downstream optimization dies
(escape analysis, GVN, loop opts, range check elimination).

| Flag | Default | Meaning | Failure mode when exceeded |
|------|---------|---------|----------------------------|
| `MaxInlineSize` | 35 bytes | *Always* inline if bytecode ≤ this | Method stays as virtual call |
| `FreqInlineSize` | 325 bytes | Inline if hot AND ≤ this | "hot method too big" → no inline |
| `InlineSmallCode` | 2000 bytes | Don't re-inline callee whose *native code* ≥ this | Late recompilation stops inlining |
| `MaxInlineLevel` | 9 | Max nested inline depth | Deep wrappers (decorators, stream pipelines) hit the wall |
| `MinInliningThreshold` | 250 | Min invocations before inlining considered | Cold startup methods miss the train |
| `NodeCountInliningCutoff` | 18000 | Stop inlining once parser generates this many IR nodes | Big methods starve later calls |
| `DesiredMethodLimit` | 8000 bytes | Aggregate post-inline bytecode cap (compile-time constant, not tunable) | Silent inlining stops — only `-XX:-ClipInlining` disables |
| `LiveNodeCountInliningCutoff` | ~40000 | Live IR node ceiling | C2 bails out of compilation entirely |

**Non-obvious consequences:**

1. **Bytecode size matters more than logical size.** `synchronized`, exception
   handlers, and generic-erased boxing all inflate bytecode without adding
   "work." A Scala for-comprehension easily blows past 325 bytes.
2. **`InlineSmallCode=2000` is measured against *previously compiled native
   code*, not bytecode.** A small-bytecode method that C2 compiled to a big
   assembly blob will refuse to inline on later recompiles.
3. **`MaxInlineLevel=9` is nested depth, not total inlines.** Reactive streams
   with deep operator chains (flatMap.map.filter.map.flatMap…) routinely hit 9.
   Counter: refactor to reduce nesting, not to reduce total operators.
4. C2 inlines **at most 2 receivers** per call site (bimorphic). The 3rd type
   turns the site megamorphic and C2 emits a vtable/itable call — no inline,
   no EA, no nothing. This is the `TypeProfileWidth=8` confusion: the profile
   *records* up to 8, but inlining *uses* only 2.

---

## Escape Analysis: Why Your Allocation Survived

HotSpot C2 EA succeeds on only **~13% of candidate methods** (Soares, OpenJDK
2021 analysis on DaCapo+Renaissance). The failure modes are specific and
well-known:

### 1. Flow-insensitive — ANY escaping path poisons all paths

```java
MyPair o = new MyPair(1, 2);
if (rareFlag) global = o;   // escapes here
return o.p1;                // escapes here TOO, because C2 is flow-insensitive
```

Graal's **Partial Escape Analysis (PEA)** handles this; C2 does not. On
Scala-heavy benchmarks Graal shows double-digit % gains over C2 purely from
PEA. If you're on Scala, Kotlin, or Clojure and EA dominates, consider
`-XX:+UseJVMCICompiler` with Graal.

### 2. Control-flow merges with distinct allocations

```java
MyPair o = new MyPair(0, 0);
if (cond) o = new MyPair(x, y);   // merge kills scalar replacement
return o.p1;
```

Even trivially. Two `new` sites flowing into one phi → EA bails. The fix is to
lift both branches into a single constructor call or hoist the condition
outside the allocation.

### 3. Interprocedural bail at 150 bytes

C2's interprocedural EA (Kotzmann–Mössenböck algorithm) runs on *bytecode*,
not the Ideal graph, and **bails on any callee method > 150 bytes**. References
escaping through such a call are conservatively marked GlobalEscape. This is
why extracting a helper method can *worsen* EA: you crossed the 150-byte line.

### 4. Iterator elimination is a house of cards

The classic `for (Thing t : collection)` iterator lives on the stack only if:
1. The collection's concrete type is monomorphic at the call site, AND
2. `iterator()` gets inlined, AND
3. `hasNext()` gets inlined, AND
4. `next()` gets inlined, AND
5. EA then runs and sees no escape, AND
6. No control-flow merge muddies the picture.

Break any link → heap allocation per loop entry. The canonical defeat:

```java
void process(Collection<Thing> c) { for (Thing t : c) ... }
// Called with ArrayList, LinkedList, HashSet → megamorphic → no inline → iterator escapes
```

Pre-Java 9, `Arrays.asList(...)` broke EA because its iterator was a
non-static inner class holding an implicit outer reference. Fixed in JDK 9
(JDK-8170372). The lesson: **inner classes should be static by default**.

### 5. Synchronization on non-escaping objects

EA enables **lock elision** — if `synchronized(obj)` never escapes, the monitor
disappears. But if EA fails for any of the reasons above, you pay full monitor
enter/exit cost. `StringBuffer` was the canonical win here; modern code should
use `StringBuilder` anyway.

---

## Megamorphic Dispatch: The Invisible Cliff

C2's inline cache is **bimorphic** (2 types). Going from 2 to 3 types at a
call site is not a 50% slowdown — it's a qualitative cliff:

| State | What happens | Typical cost |
|-------|--------------|--------------|
| Monomorphic (1 type) | Direct call, inlined, EA flows through | ~1 cycle |
| Bimorphic (2 types) | Type-switch + both inlined | ~3 cycles |
| Megamorphic (3+) | vtable/itable call, no inline | 10–30 cycles + pipeline flush + downstream opts die |

**The hidden disaster:** type profile is collected per-*bytecode-callsite*, not
per array index. A loop like `for (Base b : array) b.m()` has *one* callsite,
even if `array[0]` always holds `X` and `array[1]` always holds `Y`. All
distinct types observed across the entire execution pollute one profile.

**The unroll-by-hand trick** (Shipilev/apangin):
```java
int n = b.length;
if (n > 0) b[0].m();  // separate callsite → separate profile
if (n > 1) b[1].m();
if (n > 2) b[2].m();
```
Each manual call site gets its own type profile. Fragile and ugly but
measurably 5–10x when it wins. **Use only after profiling confirms the
original is megamorphic.**

---

## Range Check Elimination Requires a Counted Loop

C2's loop predication hoists `i >= array.length` out of the loop body — but
only for *counted loops*, which have a rigid shape:

- Loop variable is `int` (not `long`, not `Integer`)
- Stride is a compile-time constant (`i++`, `i += 2`; NOT `i += step` where
  `step` is a parameter)
- Termination test is `i < limit` with `limit` loop-invariant
- Array index is `scale * i + offset` where `scale`, `offset` are
  loop-invariant

Break any constraint and you lose:
1. Range check elimination (bounds check every iteration)
2. Loop unrolling (impossible without counted form)
3. **SIMD vectorization** (requires unrolled counted loop)
4. Strip mining for safepoint polls (JDK-8186027) — long loops will stall GC

**The cardinal sin:** `for (long i = 0; i < arr.length; i++)`. `long` loop
variable disables counted-loop recognition. Use `int` even for arrays you
believe "might" exceed 2^31 — if they will, use a nested int loop.

---

## Deoptimization Economics

Speculation is safe only because deopt is correct. It is NOT cheap:

| Phase | Cost |
|-------|------|
| Uncommon trap | ~1–10 μs (rebuild interpreter frame, unlock monitors) |
| Interpreter re-entry | 10–30x slower execution until recompile |
| Recompile | 100ms–1s wall clock (C2 queue) |
| `PerMethodRecompilationCutoff` | Default **400**. After this many recompiles of the same method, it is **permanently banned from C2** — stuck in the interpreter *forever* |

**The deopt loop** (Crankshaft's original sin, still possible in C2):
1. Method hot → C2 compiles with speculation X
2. Speculation X fails → deopt
3. Profile didn't update with new info → recompile with same X
4. Go to 2

Detection: `-XX:+PrintCompilation` shows the method being recompiled
repeatedly. `-XX:+PrintDeoptimizationDetails` shows the trap reason.

Mitigation:
- Trust the profile before speculating (check `MethodData` counts).
- Make the speculation coarser (bimorphic instead of monomorphic).
- For pathological cases, `-XX:CompileCommand=dontinline` the offending call.

---

## Diagnostic Command Cheat Sheet

```
# What got compiled and when
-XX:+PrintCompilation

# Inlining decisions (verbose)
-XX:+UnlockDiagnosticVMOptions -XX:+PrintInlining

# Why speculation failed
-XX:+PrintDeoptimizationDetails -XX:+TraceDeoptimization

# Did EA succeed?
-XX:+PrintEscapeAnalysis -XX:+PrintEliminateAllocations

# Structured log for post-processing
-XX:+LogCompilation -XX:LogFile=c2.log
# then: java -cp ... LogCompilation -i c2.log

# Force a method to behave differently (debugging only)
-XX:CompileCommand=dontinline,com.foo.Bar::baz
-XX:CompileCommand=inline,com.foo.Bar::baz
-XX:CompileCommand=exclude,com.foo.Bar::baz   # never compile
```

**The investigator's trap:** `PrintCompilation` timestamps are wall-clock, not
thread CPU time. A method showing "3571 ms" in the compile log may be 300 ms
of actual C2 thread work starved by GC or other compile tasks. Correlate with
`perf stat -t <C2_tid>` before blaming the compiler.
