---
name: torvalds-kernel-pragmatism
description: Write systems code in the style of Linus Torvalds and the Linux kernel community — pragmatic C, "good taste" data-structure design, hard rules about regressions and userspace ABI, kernel-specific idioms (container_of, goto cleanup, ERR_PTR, RCU, READ_ONCE), and bisect-friendly commit discipline. Use when writing or reviewing Linux kernel code, drivers, kernel modules, lockless/concurrent C, or any high-stakes systems code where correctness, latency, and reviewability dominate. Triggers: "kernel patch", "kernel module", "device driver", "Linux kernel coding style", "kernel C", "RCU", "spinlock", "GFP_KERNEL/GFP_ATOMIC", "container_of", "goto cleanup", "ERR_PTR", "memory barrier", "READ_ONCE", "WRITE_ONCE", "lockless", "submit to LKML", "checkpatch", "subsystem maintainer", "git bisect", "Signed-off-by", "kernel review", "torvalds style".
---

# Torvalds / Linux Kernel Pragmatism⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌​‌​‌‌‌​‍​​‌​‌​​​‍‌‌‌​‌​‌‌‍​‌‌‌​‌‌‌‍​‌‌​​‌​‌‍‌​‌​​‌‌‌‍​‌‌​‌‌‌‌‍​​​​‌​‌​‍​‌​‌‌‌​​⁠‍⁠

## The mental model

When you write code in this style, you are not "writing a function". You are placing one more block in a structure that **must remain bisectable, revertable, and runnable on every machine that ran it last week**. Three questions decide nearly everything:

1. **Where are the special cases?** Special cases are bugs waiting for an input you didn't think of. Restructure the data so the special case becomes the normal case (see `references/philosophy.md`, the indirect-pointer technique).
2. **Could this break a userspace program that worked yesterday?** If yes, the answer is no — the kernel's bug is the kernel's to fix, not the user's. There is no "but the old behavior was wrong" exception.
3. **Can a tired maintainer at 2am revert just this commit and have a working tree?** If your patch only makes sense paired with three other commits, split it again or reorder until each commit compiles, boots, and passes its own tests.

If you cannot answer all three before writing the patch, you are not ready to write the patch.

## Decision rules that take years to internalize

- **3 levels of indentation is the warning siren, not the limit.** When you reach the 4th `if` inside a `for` inside a `while`, the bug is not the indentation — it's that the function is doing something the data structure should be doing. Fix the structure, not the brace.
- **Functions over ~40 lines or with >5–10 locals are usually two functions.** A human keeps ~7 things in working memory; longer functions silently exceed that and you stop seeing the bug even when it's on screen.
- **Optimize for the reviewer, not the writer.** Code is written once and read 50 times by people who are angry, sleep-deprived, and bisecting a regression. Every "clever" trick costs them five minutes; multiply by 50.
- **`likely()`/`unlikely()` are not for "I think this is faster".** They are only correct when the branch ratio is >99/1 (e.g. error paths, slowpath fallbacks). Wrong hints are worse than none — modern branch predictors learn the actual ratio in microseconds, but the compiler emits the wrong layout permanently.
- **Cache misses dominate over instruction count below ~100 cycles of work.** A pointer chase across a list is ~100–300 cycles per node on cold cache. An array scan of 16 elements is one cache line. "Linked list vs array" is almost never about big-O at small N.
- **`inline` keyword is mostly noise.** GCC inlines static-used-once functions itself. Hand-inlining anything >3 lines just defeats the maintainer who wants to remove it when it gets a second caller. Exception: macros that depend on `__builtin_constant_p` of an argument.
- **No raw `kmalloc(sizeof(struct foo), ...)`.** Always `kmalloc(sizeof(*p), ...)` against the destination variable. When `struct foo` is renamed, the type-based form silently allocates the wrong size; the variable-based form breaks the build.

## Anti-patterns — never, and the non-obvious reason why

- **NEVER add a `prev == NULL` (or `i == 0`, or `head->next == NULL`) special case in a list/tree/buffer routine.** It is seductive because it "just handles the edge". The real consequence: you have doubled the paths a reviewer has to verify, and the bug they don't catch will be in the special branch because nobody hits it in testing. Instead, restructure with the indirect-pointer / sentinel / dummy-head technique in `references/philosophy.md`.
- **NEVER write `if (a) { ... } else { ... }` where both branches do the same store with a different LHS.** It's tempting because it "reads naturally". The consequence: the duplication hides the actual algorithm and rots the moment one branch needs an extra step. Lift the variable LHS into a pointer-to-pointer and do one assignment.
- **NEVER use `typedef` for a struct just to save typing `struct`.** It looks cleaner. The consequence: a reader can no longer tell from the call site whether they're holding a value, a pointer, or an opaque handle, and review becomes guesswork. The kernel forbids it (`CodingStyle` ch.5) except for genuinely opaque types where hiding the layout is the *point* (`pid_t`, `sector_t`).
- **NEVER call a sleeping function (`kmalloc(GFP_KERNEL)`, `mutex_lock`, `copy_from_user`, `msleep`) inside `rcu_read_lock()`, a spinlock, an interrupt handler, or any code that ran from `softirq` / NMI / `preempt_disable()`.** The compiler will not stop you. Production will: the box deadlocks under load when reclaim recursively tries to take the lock you're holding. Use `GFP_NOWAIT` in atomic context, `GFP_ATOMIC` only when failure would be worse than depleting reserves. Decision tree in `references/concurrency.md`.
- **NEVER touch shared memory without `READ_ONCE`/`WRITE_ONCE` (or stronger).** A plain `x = shared` looks identical to the compiler as scratch storage; it is allowed to load it byte-by-byte, refetch it twice, or speculate a value. The bug appears once a month under load and is unreproducible. See `references/concurrency.md`.
- **NEVER do "trivial" mass cleanup conversions** (`strncpy`→`strscpy`, renaming an API across hundreds of files, reformatting). The consequence Linus has called out by name: each file looks fine, nobody reviews 400 files carefully, and one of them silently flips a return-value sign or truncates a length. New API for new code; convert old code only with a real bug as justification.
- **NEVER write `if (ret < 0) return ret;` ten times in a function with allocations.** It leaks every resource acquired before the failing call. Use the `goto err_X:` cleanup ladder, unwinding in *reverse order* of acquisition. Pattern in `references/kernel-idioms.md`.
- **NEVER rebase or force-push a branch you've published.** History is the bisect substrate. Rewriting it makes every downstream tree's bisect lie. Local cleanup before publishing is fine; after `git push`, the history is frozen.
- **NEVER mark a commit `Cc: stable@` without testing the build on the actual stable branch.** Stable backports that "looked obvious" but never compiled on the older tree are one of Linus's most-quoted rant categories.
- **NEVER use floating point in kernel code** (no `float`, no `double`, no `printf("%f")`, no SIMD without `kernel_fpu_begin()`/`_end()`). The FPU state belongs to userspace and the kernel doesn't save/restore it on entry. You will silently corrupt a userspace process's registers.

## Procedures Claude wouldn't already know

### Before writing a new kernel function, in this exact order:
1. Sketch the **struct** the function operates on. If you can't draw it on paper, you don't understand the problem yet.
2. Write the **error-unwind ladder first** (the `err_*:` labels at the bottom). This forces you to enumerate every resource the happy path acquires before you write the happy path.
3. Decide the **calling context**: process / softirq / hardirq / NMI / RCU read-side / spinlock-held. Write it in a comment above the function. This determines every allocation flag and every primitive you're allowed to call.
4. Only then write the body.

### Before submitting a patch series:
1. `git rebase -i` so every commit **builds and boots independently** — `git bisect` will land on each one. A series of 8 commits where commits 1–7 don't compile is a series of 1 commit pretending to be 8.
2. `scripts/checkpatch.pl --strict` on every patch. Not because checkpatch is right (it isn't always), but because reviewers will run it and you don't want the conversation to be about whitespace.
3. Each patch's commit message answers **why**, never **how**. The diff is the how. The first line is `subsystem: imperative summary <=50 chars`; blank line; body wrapped at 72; trailers (`Fixes:`, `Reported-by:`, `Signed-off-by:`) at the bottom. Format details in `references/userspace-contract.md`.
4. For any change touching syscalls, `/proc`, `/sys`, ioctl, or netlink: stop and re-read `references/userspace-contract.md`. The rules there are non-negotiable.

## When to load which reference

**MANDATORY** — read the matching reference *before* writing code in these scenarios. Each is short (~100–200 lines) and self-contained.

| You are about to... | READ |
|---|---|
| Eliminate a special case, design a list/tree/buffer API, or feel the urge to add an `if first_element` branch | `references/philosophy.md` |
| Write a function that does ≥2 allocations or acquires ≥2 resources | `references/kernel-idioms.md` (goto cleanup ladder, ERR_PTR, container_of, GFP flag decision tree, kref) |
| Touch any data shared between CPUs, threads, or interrupt context | `references/concurrency.md` (RCU rules, READ_ONCE/WRITE_ONCE, memory barriers, atomic-context table) |
| Change a syscall return value, add a new ioctl, modify `/proc` or `/sys` output, write a commit message, or split a series for submission | `references/userspace-contract.md` |

**Do NOT load** every reference reflexively. Loading `concurrency.md` for a pure-userspace bug fix is wasted context. Loading `userspace-contract.md` to write a new helper function is wasted context. Match the file to the actual task.

## Fallbacks when the rules don't fit

- **Userland C, not kernel:** keep the data-structure-first instinct, the goto-cleanup pattern, the commit discipline, and the special-case elimination. Drop the `GFP_*` / `READ_ONCE` / RCU machinery — you have malloc, mutexes, and a real OS underneath.
- **You inherited a 200-line function with 6 indentation levels and can't restructure it now:** at minimum, extract the deepest block into a helper with a descriptive name. Don't reformat — that destroys `git blame`. Add a `FIXME(torvalds-style):` so the next reader knows you saw it.
- **A reviewer says "this is unmaintainable" but won't say what:** the answer is almost always *too many special cases* or *the wrong data structure*. Ask "what would this look like if `prev` didn't exist?" or "what if there were always at least one element?" — the elimination usually falls out.
