# The Userspace Contract — and how to ship a patch that survives review⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​‌‌​​​‍‌​‌​​​‌‌‍​‌‌‌‌‌​​‍​‌​‌‌​‌​‍​​​​‌​​‌‍‌​‌​‌​​‌⁠‍⁠

Load this when you are about to change anything observable from userspace, write a commit message, or split work into a series for submission.

## Rule zero: WE DO NOT BREAK USERSPACE

The kernel makes exactly one promise to the rest of the world: **a userspace program that ran on Linux N will run on Linux N+1**, regardless of how ugly the old kernel's behavior was, how clearly the old behavior was a bug, how much cleaner the new code is, or how few users you think depend on it.

Concretely, this means the following are immutable contracts once shipped in a stable release:

- **Syscall numbers, argument types, return value semantics, and the exact set of errno values returned for each input.** Returning `-EINVAL` where you used to return `-ENOSYS` is a regression.
- **`/proc` file output format**, including column order, whitespace, decimal-vs-hex, and trailing newlines. People grep this stuff in shell scripts.
- **`/sys` attribute output format**, same reasoning. Sysfs is "one value per file" partly so format changes are constrained.
- **ioctl numbers, struct layouts, struct sizes, padding bytes.** Adding a field to the end of a struct is OK *only if* old binaries that pass the old size still work; check `_IOC_SIZE` handling.
- **Netlink message formats, including attribute order in some cases.**
- **Module parameter names** (anything in `/sys/module/.../parameters/`).
- **`printk` message text that contains numbers or paths**, because `dmesg | grep` and log parsers exist. (Free-form text is mostly safe; structured-looking text isn't.)
- **The set of files in `/dev` that a stable driver creates.**

What is *not* a userspace ABI and may be broken freely:

- **In-kernel APIs** (`EXPORT_SYMBOL`). Out-of-tree modules are not Linus's problem; they are expected to keep up.
- **`/sys/kernel/debug/...`** (debugfs). It is explicitly unstable and userspace tools that depend on it are wrong.
- **The undocumented behavior of `printk` log levels and ratelimiting.**
- **Internal data structure layouts**, even if they accidentally became visible via `/proc/kcore` or BPF.

### The test for "is this a userspace regression?"

> Did *some* userspace program work before this commit and stop working after it?

If yes: **it is a regression, period.** Not "it's a regression unless the old behavior was buggy". Not "it's a regression unless the program was relying on undocumented behavior". Not "it's a regression unless we warned in the changelog". The kernel reverts. The user does not have to upgrade their program. This is the entire reason Linux is the kernel of everything.

The famous Torvalds quote, verbatim, because it is the rule:

> "If a change results in user programs breaking, it's a bug in the kernel. We never EVER blame the user programs."

The corollary that's easy to miss: **a "fix" that introduces a regression is not a fix.** The original bug is preferable to the regression. Revert first, find a non-regressing fix later.

## Bisect-friendly commits — what "small" actually means

`git bisect` is the kernel's main debugging tool. A bisection over 10,000 commits is 14 steps; over 100,000, it is 17. This works **only if every commit in the history compiles, boots, and runs the relevant test**. If even 5% of commits are broken, bisection becomes useless because you keep landing on `bisect skip`.

This is why the rules are:

- **Each commit must compile on every supported architecture** that the changed code touches. Not "the series compiles" — *each commit*.
- **Each commit must be a logically complete change.** "Add helper" + "use helper" is fine as two commits if both build. "Rename function across 30 files" must be one commit, not 30 — partial renames don't build.
- **Do not mix unrelated changes in one commit**, even tiny ones. The whitespace fix and the bug fix go in separate commits. When the bug fix is later reverted for causing a regression, you don't want to lose the whitespace fix with it.
- **Refactors that don't change behavior should be marked clearly** in the commit message ("No functional change intended" / "NFC"). This lets reviewers skim them and lets bisection skip them faster.
- **`git rebase -i` your branch before publishing** to squash "fix typo", "address review", "really fix it this time" commits into the logical commit they belong to. Once published, history is frozen.

## The kernel commit message format — every line matters

```
subsystem: imperative summary in <=50 chars

Free-form body wrapped at 72 columns explaining WHY this change is
needed, not HOW it works. The diff is the how. The reader will look
at the diff anyway; the commit message is your one chance to
explain the motivation, the alternatives you considered, the
constraint that ruled them out, and any subtlety future-you will
forget in six months.

If there is a reproducer, paste the exact command. If there is a
crash, paste the relevant lines of the splat. If this fixes a
specific commit, cite it.

Fixes: 1234abcd5678 ("subsystem: original buggy commit summary")
Reported-by: Real Name <real@email>
Tested-by: Real Name <real@email>
Reviewed-by: Real Name <real@email>
Cc: stable@vger.kernel.org # v5.10+
Signed-off-by: Your Real Name <your@real.email>
```

Non-obvious rules:

- **Subject line is imperative mood** ("Fix the leak", not "Fixed the leak" or "Fixes the leak"). Matches what `git` itself produces ("Merge branch...").
- **Subject line starts with the subsystem prefix**, lowercase, colon-separated. Look at `git log --oneline path/to/file/you/changed` to see the local convention. Wrong prefix is the #1 reason a patch gets ignored.
- **No "this patch", "this commit", "I", "we"** in the body. The reader knows it's a patch. Use the imperative throughout: "Add X to handle Y" not "This patch adds X".
- **`Fixes:` tag uses 12-character SHA + the *exact* original subject line in parentheses**, no editing. There are scripts that parse this.
- **`Cc: stable@...` is a *request*, not a guarantee.** It tells the stable team "this should be backported"; the format `# v5.10+` says "to 5.10 and later". Without testing on the stable branch first, do not add this line.
- **`Signed-off-by:` is a legal statement** (the Developer Certificate of Origin) that you have the right to submit this code under the kernel's license. It is not optional and not decorative. `git commit -s` adds it automatically.
- **Never break user-visible strings** (printk text, error messages) across lines for the 72-column rule. People grep them. Let the line be long.

## Patch series — how to split, in what order

A series is a sequence of commits posted together. Reviewer attention is the bottleneck, so:

- **Patch 1 of N is usually the boring prep** (rename a thing, extract a helper, export a symbol). Reviewers can ack it in 30 seconds.
- **Patches 2..N-1 are the meat**, each one adding one logical capability.
- **The last patch enables the feature** (Kconfig change, callback registration, default flip). This makes the series easy to revert: drop the last patch and the rest is dormant.
- **Do not put the test/benchmark in the same patch as the change.** Separate commit so it can be re-run independently.
- **Cover letter (`0/N`) explains the *series*, not any one patch.** What problem, what design, what alternatives, what testing. Each patch's own message explains its own role.

If your series is more than ~10 patches, you are usually trying to do too much in one go. Split it into independent series that can be merged separately.

## Things Linus has personally rejected, and the lesson from each

- **Mass mechanical conversions** (`strncpy → strscpy` over hundreds of files): rejected because they get rubber-stamped, the silent bug rate is non-zero, and the conversion makes `git blame` worse. **Lesson:** use new APIs in new code; convert old code only when fixing a real bug there.
- **"Fix" patches that change `errno` values returned from a syscall**: rejected because some userspace program relies on the old value. **Lesson:** before changing any return value, ask "what would happen if a script greps for the old number?"
- **Patches that "clean up" by removing #ifdef blocks for old compilers/architectures**: sometimes accepted, sometimes rejected — depends on whether the architecture has any users. **Lesson:** removing support is a separate process from cleanup, and requires evidence (usually a year on a deprecation list).
- **Code marked for `stable@` that didn't compile on the stable branch**: rejected with prejudice. **Lesson:** if you tag stable, build-test on the actual stable branch first. The stable maintainer's time is not yours to waste.
- **Patches whose subject line is not in the subsystem's house style**: silently dropped. **Lesson:** read 20 lines of `git log` for the file you're changing before writing your subject line.

## The minimum pre-submission checklist

1. `make C=2 W=1` over the changed files — no new sparse warnings, no new compiler warnings.
2. `scripts/checkpatch.pl --strict` on every patch — fix the real complaints, document the false-positive ones in the cover letter.
3. Boot a kernel with the patch applied. Yes, even for "trivial" patches. Especially for trivial patches.
4. Run any selftest under `tools/testing/selftests/` that touches the changed subsystem.
5. `git rebase --exec 'make -j$(nproc)' base..HEAD` — every commit builds.
6. Re-read your own commit messages as if you were a reviewer who has never seen the code. If "why" is not obvious, rewrite.
7. Send with `git send-email`, plain text, to the maintainer + mailing list listed in `MAINTAINERS` for the changed file. HTML mail and attachments are auto-discarded by some maintainers.

If any step in this list feels like overkill for "such a small patch", that is exactly the patch where the rule will catch a real bug. The rules exist because they each came from an actual disaster.
