---
name: torvalds-kernel-pragmatism
description: "Kernel-maintainer heuristics for Linux C, review, and patch flow: regression triage, bisect-safe patch slicing, calling-context decisions, UAPI immutability, and review-scarred anti-patterns. Use when writing or reviewing Linux kernel code, drivers, modules, lockless/RCU/IRQ paths, ioctl/sysfs/proc/syscall changes, or LKML/stable-ready patch series. Triggers: kernel patch, device driver, kernel module, LKML, stable backport, regression, RCU, spinlock, GFP_ATOMIC, READ_ONCE, ioctl, sysfs, procfs, Fixes tag, checkpatch, git bisect."
---

# Torvalds / Kernel Pragmatism

Treat kernel work as damage control on a live ecosystem, not as a greenfield coding exercise. The winning move is usually the one that preserves userspace, preserves bisectability, and lets an annoyed maintainer verify the change without reconstructing your intent from scratch.

## Before you touch code, ask yourself

- **Is this a regression?** If yes, the default is revert-first, not clever-fix-first. The regression docs explicitly say to always consider reverting the culprit, to aim for a fix within 2-3 days when impact is severe, and to avoid letting even current-cycle regressions drift to the end of the cycle.
- **Is anything userspace-visible?** Syscalls, `ioctl`, `/proc`, `/sys`, netlink payloads, module parameter names, and structured `printk` output are contracts. Internal kernel APIs are disposable; userspace ABI is not.
- **What exact context am I running in?** Process vs softirq vs hardirq vs NMI vs RCU read-side is not an implementation detail; it determines whether sleeping, locking, and allocation are legal at all.
- **Can each commit stand trial alone?** If `git bisect` lands on commit N, that commit must build, boot, and justify itself without commit N+1.
- **Am I solving one real bug, or am I shipping review bait?** Tree-wide cleanups, style churn, and mass API conversions consume reviewer budget while reducing signal.

## Heuristics that matter more than style

- Stable backports are deliberately narrow: upstream first, obviously correct, tested, and typically no bigger than 100 lines including context. A fix that needs a long verbal defense is usually not a `stable@` candidate yet.
- `GFP_ATOMIC` is not "safer kmalloc"; it burns atomic reserves. The MM docs also note that current `GFP_ATOMIC` handling does not cover NMI or contexts that disable preemption under PREEMPT_RT such as `raw_spin_lock()` and plain `preempt_disable()`.
- `__GFP_NOFAIL` is only honest when the caller truly has no failure policy, may sleep indefinitely, and is not asking the buddy allocator for order > 1. For larger "must succeed" buffers, the docs point you at `kvmalloc()` or a redesign.
- Plain shared-memory accesses are wrong even on x86. The compiler can fold, refetch, or tear them. For add-only RCU structures, `READ_ONCE()` can be enough; once removal or lifetime changes enter the picture, use `rcu_dereference()` / `rcu_assign_pointer()`.
- Kernel stacks are small enough to be a design constraint: about 3K-6K on many 32-bit configurations and about 14K on many 64-bit ones, often shared with interrupt handling. VLAs are banned because they generate worse code and can walk off the remaining stack budget.
- `BUG()` is not a tougher `WARN()`. It destabilizes the machine and can destroy the evidence needed to debug the real problem. Kernel docs now say new code should use `WARN*()`, usually `WARN_ON_ONCE()`, with recovery when possible.
- UAPI padding is future budget. If you do not reject non-zero unknown fields and padding, random userspace stack garbage becomes part of the ABI forever and you lose extension room.
- A `Fixes:` tag needs at least 12 hex chars plus the exact original one-line summary. The same docs say not to split the tag across lines because scripts parse it.
- If you move code between files, do not modify it in the same patch. Kernel submission docs call this out because mixed move+edit commits destroy history tracking and force reviewers to diff noise instead of behavior.
- For non-trivial series, use `git format-patch --base=auto --cover-letter`. The `base-commit:` trailer is not decorative; it tells reviewers and CI exactly which tree to replay instead of guessing from your branch name.
- Review throughput drops off hard after about 15 patches in flight. If the series cannot be condensed, post in chunks instead of asking maintainers to hold the entire graph in their heads.
- Reviewers trust explicit interleavings more than adjectives. For races or ordering bugs, draw the CPU0/CPU1 timeline in the changelog instead of writing "obviously synchronized".

## NEVER do the seductive thing

- **NEVER send tree-wide mechanical conversions** because they look low-risk, reviewers skim them, `git blame` gets noisier, and the one semantic drift hidden in file 173 survives to production. Instead convert only bug-adjacent code or new call sites, and justify each old-site conversion with a real defect.
- **NEVER rebase published or borrowed history** because reparenting invalidates prior testing and poisons downstream bisects; rebasing onto a random in-between kernel commit is especially bad. Instead clean up only private branches and, if rebasing is unavoidable, move to a stable point such as a release or `-rc` and retest from scratch.
- **NEVER use `volatile` for shared kernel state** because it suppresses optimization inside already-correct critical sections while leaving the real race untouched. Instead encode the concurrency model with locks, `READ_ONCE`/`WRITE_ONCE`, release/acquire operations, or RCU primitives.
- **NEVER open-code allocator arithmetic** because overflow becomes an undersized allocation and the eventual heap corruption happens far from the call site. Instead use `kmalloc_array()`, `kcalloc()`, `struct_size()`, and `size_*()` helpers.
- **NEVER treat `GFP_ATOMIC` as generic defensive coding** because it consumes emergency reserves and still does not make sleeping callers legal. Instead identify the real context and choose `GFP_KERNEL`, `GFP_NOWAIT`, a preallocated pool, or a different call path.
- **NEVER add or extend an `ioctl` without fixed-width types, explicit padding, zero checks, and feature discovery** because ignored tail fields and garbage padding become ABI commitments. Instead design for 32/64-bit compat on day one and reject unknown bits immediately.
- **NEVER mark a patch `Cc: stable@vger.kernel.org` just because it feels harmless** because stable rules explicitly reject theoretical races and oversized or under-tested fixes. Instead land upstream first, prove user impact, and test against the actual stable branch you want to target.
- **NEVER use `BUG_ON()` as a shortcut for error handling** because the seductive part is that it hides unwind code, but the consequence is a less debuggable and sometimes unrecoverable system. Instead use `WARN_ON_ONCE()` for truly impossible states or unwind and return an error.

## Mandatory routing into the local references

| Before you do this | Read this first | Do not load this for that task |
|---|---|---|
| Touch shared state, ordering, RCU, IRQ/softirq interactions, or lockless polling | `references/concurrency.md` | Do not load `references/userspace-contract.md` unless the change is also userspace-visible |
| Add allocations, pointer-or-error returns, refcounts, or multi-step cleanup | `references/kernel-idioms.md` | Do not load `references/philosophy.md` unless the core problem is data-shape or special-case elimination |
| Feel tempted to write `if (first)`, `if (!prev)`, dummy one-off head/tail logic, or duplicate branches with different LHS | `references/philosophy.md` | Do not load `references/concurrency.md` for a purely structural refactor |
| Change a syscall, `ioctl`, `/proc`, `/sys`, netlink format, commit tags, series splitting, or backport intent | `references/userspace-contract.md` | Do not load `references/kernel-idioms.md` for a commit-message-only task |

## Decision tree when the first idea feels "too big"

1. If a regression has a safe revert, ship the revert first and the improved version later.
2. If the fix is too large for stable or depends on refactoring, split it into: minimal user-visible fix now, cleanup later.
3. If a race explanation needs adjectives like "subtle" or "probably safe", stop and write the event timeline; if you cannot explain the ordering in columns, you do not understand it yet.
4. If the patch changes both behavior and cleanup, separate them unless the cleanup is mechanically required for the behavior change.
5. If this is userland C rather than kernel code, keep the no-regression mindset, bisect-friendly slicing, overflow-safe allocation helpers, and cleanup ladders; drop the kernel-only GFP/RCU/UAPI rules.

## Done looks like this

- The patch solves one real problem and leaves unrelated cleanup for later.
- The calling context is explicit enough that sleeping/allocation/locking legality is obvious.
- Any userspace-visible change was treated as ABI work, not as an internal refactor.
- Every commit builds and can be reverted independently.
- The references above were loaded only when the task actually needed them.
