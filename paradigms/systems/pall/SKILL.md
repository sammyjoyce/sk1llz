---
name: pall-jit-mastery
description: "Mike Pall-style trace-JIT diagnosis and LuaJIT/FFI tuning for hot dynamic-language runtimes. Use when a path flips between interpreted and native code, guard or side-exit churn dominates, FFI callbacks or C boundaries poison traces, or you need to decide whether to reshape code, retune hotloop/hotexit, or disable JIT for one function. Trigger keywords: luajit, trace abort, side trace, guard, -jv, -jdump, -jp, hotloop, hotexit, maxrecord, maxsnap, mcode, bad callback, ffi."
---

# Pall JIT Mastery

## Use this skill for

- Trace-JIT triage in LuaJIT-style runtimes where tiny source edits can flip whole execution modes.
- FFI boundary design when "cleaner C code" is tempting but traceability matters more than language purity.

Do NOT load generic JIT/compiler primers for this task. This skill assumes you already know what a trace, guard, and side exit are.

## Before you change anything

Before touching code shape or runtime flags, collect all three from the target workload:

- `-jp=vf` or `-jp=fv` to split time by VM state: `N` native, `I` interpreted, `C` C code, `G` GC, `J` JIT compiler.
- `-jv` to confirm whether roots and side traces are actually starting.
- The first meaningful abort from `-jdump`; the first real abort is usually worth more than the fiftieth.

Before doing X, ask yourself:

- Is this a capacity problem, a churn problem, or a foreign-boundary problem?
- Am I reducing the number of guards the trace must prove, or only moving them around?
- If I change a knob globally, what cold code will now become hot enough to waste compile budget?
- If I cross into C here, can the recorder still inline, hoist, sink, and specialize, or did I just freeze the optimization boundary?

## Operating stance

- Treat traces as speculative contracts. A fast trace is not "optimized code"; it is a narrow bet that the same types, branches, and layouts recur.
- The highest-leverage fix is usually reducing assumption surface, not writing cleverer code.
- One unstable guard can be worse than a slower algorithm because it keeps paying interpreter re-entry, hot-exit counting, and re-recording costs.
- Raising limits is the last move. First prove whether the trace budget is too small or whether the program is feeding it unstable shape.

## Triage by symptom

### 1. `-jp` says time is mostly `I`

- Meaning: the hot path never became trace-worthy, was blacklisted, or keeps aborting before stabilization.
- Check `-jv`. If it stays quiet, verify hotness and JIT enablement before rewriting code.
- If roots start and then disappear, inspect the first abort, not the aggregate sample.

### 2. `-jp` says time is mostly `J`

- Meaning: compile churn. You are spending time recording and assembling rather than executing.
- Common causes: `hotloop` too low, side exits being retried, repeated flush-retrace loops, or one mega-trace hitting `maxrecord` or `maxsnap`.
- Remedy: simplify the kernel first. Tuning comes second.

### 3. `-jp` says time is mostly `C`

- Meaning: the foreign boundary is the bottleneck, or the optimizer cannot see through it.
- Default move: pull the tight loop back into Lua/IR space. LuaJIT can inline a Lua callback into an integration loop; it cannot inline C calling back into Lua.
- If the C API is fixed, prefer pull-style APIs over push-style callbacks.

### 4. `-jp` says time is mostly `G`

- Meaning: allocation sinking failed or the hot path allocates on every iteration.
- Fix object lifetime and temporary structure churn before micro-tuning arithmetic.

## First abort decoder

- `NYI: bytecode ...`: the recorder cannot encode an observed operation. Knob tuning will not save this path. Move the operation out of the hot loop or change the representation it sees.
- `NYI: unsupported C function type`: the seductive move is "just wrap it in C." The usual result is a permanently opaque call boundary. Keep the hot part in Lua or push the call onto a cold edge.
- `trace too long` or `too many snapshots`: you are mixing hot kernel logic with branchy or stateful control flow. Split the kernel; do not hand-unroll harder.
- `machine code too long` or `hit mcode limit`: first ask whether one trace is obese or whether the program has many stable traces. Only the second case justifies more cache.
- `bad callback`: stop debugging the symptom. The surrounding Lua function must not run JIT-compiled if that C call can re-enter Lua.

## Knobs that matter, and what they actually mean

- `hotloop=56`: a warmup threshold, not a speed knob. Lower it only when stable kernels are short-lived and startup latency matters. Lowering it globally pulls noise into the recorder.
- `hotexit=10`: a side-trace promotion threshold. Lowering it is seductive when exits look expensive, but it also turns rare mispredictions into compiled artifacts.
- `tryside=4`: a stop sign. If a side trace cannot stabilize within four attempts, treat the path as semantically unstable until proven otherwise.
- `maxrecord=4000` and `maxsnap=500`: almost always a shape problem before a capacity problem. Large traces usually mean mixed hot and cold logic, not a compiler that needs "more room".
- `maxtrace=1000`, `maxside=100`, `sizemcode=64`, `maxmcode=2048` in KB: only raise these after proving you have many independent stable kernels. If one kernel is churning, bigger caches mostly preserve a larger mistake.
- `-Ofma`: off by default for a reason. It trades determinism for speed and changes floating-point behavior. Never flip it on casually in finance, simulation, or test-sensitive workloads.
- OpenResty's LuaJIT fork raises `maxtrace`, `maxrecord`, `maxmcode`, and sets `minstitch=3` for very large applications. That is a capacity workaround for huge stable programs, not a first-line answer to abort churn.

## Hidden mechanics practitioners forget

- The hot-penalty cache in 2.1 source is small (64 slots) and penalties can ramp from about 72 toward 60000. Repeated aborting sites are deliberately backed off. If you keep flushing and retracing the same pathological loop, you may benchmark penalty behavior instead of steady-state behavior.
- `jit.on()` and `jit.off()` set compile eligibility; they do not force immediate compilation. Using them as a "compile now" switch is a category error.
- `jit.flush(trace)` only flushes a root and its side traces; linked code can stay live. A flush-based benchmark reset can be partial.

## FFI and boundary traps

- Callback resource ceilings are real: only about 500-1000 callbacks can exist at once, depending on architecture. Implicit callback conversions are permanent, anchored, and unreclaimable until process exit.
- A JIT-compiled FFI call that later calls back into Lua can panic with `bad callback` if the interpreter heuristic missed it. Message-polling APIs are the classic trap. If the C call may eventually re-enter Lua, put `jit.off()` around the surrounding Lua function instead of hoping the heuristic saves you.
- Vararg C functions default Lua numbers to `double`. If the callee expects `int`, it can see garbled or uninitialized data. The only legitimate reason to box scalars with `ffi.new("int", x)` is overriding vararg conversion.
- Boxing scalars with `ffi.new()` or `ffi.cast()` does not force cheap integer math. It adds boxing and unboxing overhead and cdata arithmetic becomes 64-bit sticky, which can silently change comparison and shift behavior.
- Strict aliasing is enforced even for `char *` accesses. Type punning by cast is unsafe territory; if you need punning, use a declared `union`, which LuaJIT detects and allows.
- Cdata table keys hash by address, not value. `t[1LL+1LL]`, `t[2LL]`, and `t[2]` are different lookups. Convert to number or string keys, or build a dedicated by-value hash table.
- Pointers do not keep the pointed-to cdata alive. `ffi.new("foo_t", ffi.new("int[10]"))` creates a stale-pointer bug on the next GC cycle.

## NEVER do these

- NEVER lower `hotloop` globally because a benchmark looks under-jitted. That is seductive because it produces quick wins on the demo path. Instead, first prove the target kernel is stable enough to deserve earlier compilation.
- NEVER lower `hotexit` to paper over branch instability, because it manufactures side traces for noise and burns compile budget on rare paths. Instead, remove the polymorphism causing the exits or isolate that branch as cold code.
- NEVER respond to `maxrecord` or `maxsnap` by immediately raising the limits, because the seductive story is "the compiler almost had it." The usual consequence is a larger trace with worse spill pressure and code-cache churn. Instead, split the kernel and reduce live state.
- NEVER wrap hot logic in C callbacks because the API looks elegant. The concrete cost is no cross-language inlining, slow C-to-Lua transitions, anchored callback resources, and occasional `bad callback` panics. Instead, keep the loop in Lua and call C in a pull-style way.
- NEVER use `ffi.cast()` or `ffi.new()` on scalars to "help the JIT", because it feels explicit and low-level. The consequence is extra conversions plus 64-bit-sticky arithmetic surprises. Instead, stay in plain Lua numbers unless you are overriding vararg conversion or matching a precise ABI boundary.
- NEVER use cdata values as table keys because they look like exact machine values. The consequence is address-based hashing, silent misses, and impossible cache behavior. Instead, canonicalize to Lua numbers or strings, or own the hash table yourself.
- NEVER assume `char *` can safely alias anything, because C programmers learn that habit early. LuaJIT's JIT does not honor that C99 escape hatch. Instead, use a union when you truly need type punning.
- NEVER benchmark after repeated `jit.flush()` loops without checking `-jp=vf`, because it feels like a clean reset. The consequence is measuring compile churn and penalty backoff instead of steady-state execution. Instead, validate whether traces actually restabilize after the flush.

## Freedom calibration

- High freedom: choosing trace boundaries, deciding whether to split kernels, and reshaping data to reduce guard surface.
- Low freedom: ABI declarations, callback lifetimes, global `-Oparam` changes, and any benchmark claim. For those, make one change at a time and keep a rollback path.

## When the primary move fails

- If the hot path stays interpreted after cleanup, disable JIT on exactly that function and measure again. If performance barely changes, you were chasing the wrong hotspot.
- If raising cache limits helps, verify the win survives a long run. Temporary wins often come from postponing eviction, not fixing instability.
- If moving work into Lua makes it faster, do not be surprised. Under Pall-style tracing, "higher-level" source often wins because the recorder can see and specialize the whole composition.

This style is guard-first, boundary-aware, and hostile to fake wins. Small semantic changes can flip entire execution modes; optimize the assumptions before you optimize the instructions.
