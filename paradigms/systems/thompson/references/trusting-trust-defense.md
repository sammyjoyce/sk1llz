# Trusting Trust: The Actual Defenses⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌​​​‌‌‍​​​​​​‌‌‍​‌‌‌​‌​​‍‌​‌‌​‌​‌‍​​​​‌​​‌‍‌‌‌​​‌‌‌⁠‍⁠

Load this file when: auditing a toolchain, signing a release, evaluating a supply-chain attack claim, designing a bootstrap process, or discussing "but we reviewed the source."

## What Thompson actually proved

In 1984, Thompson's Turing Award lecture described (and in 1975, actually built) a **99-line C source patch plus a 20-line shell script** that does three things in one compiler binary:

1. When compiling `login.c`, insert a back door accepting a magic password.
2. When compiling `cc.c` (the compiler itself), insert the code that does #1 and #2.
3. Leave the compiler source code pristine.

Once the poisoned binary has compiled a clean compiler source once, the backdoor is baked into the new binary and the source stays clean forever. **No amount of source review will find it**, because there is nothing to find in source.

Russ Cox ran Thompson's original code in a V6 Unix simulator in 2023. The shocking part is how small it is. You do not need nation-state resources; you need a weekend and the willingness to do it.

**Scope of the attack:** not just C compilers. Any "program-handling program" — assembler, linker, loader, firmware flasher, microcode updater, package installer — can carry the same self-replicating payload. "Recompile from source" is not a defense against any of them.

## The only known defense: Diverse Double Compiling (DDC)

David A. Wheeler's 2005 PhD thesis formalized the only known reliable defense: **build the suspect compiler's source twice — once with the suspect compiler, once with an independently-written compiler — and compare bit-for-bit.** This is DDC.

### The algorithm (4 steps, all required)

Let `A` = the suspect compiler binary. `S_A` = its source code. `T` = a trusted, independently-developed compiler (a "second opinion").

1. **Compile `S_A` with `A`** → produce `A₁`. If `A` is honest, `A₁ ≡ A` (modulo timestamps). If `A` has a Thompson payload, `A₁` is also poisoned (the payload reinserts itself).
2. **Compile `S_A` with `T`** → produce `A_T`. `A_T` is semantically equivalent to `A₁` if `S_A` is clean, but will have different instruction sequences because `T` and `A` generate code differently.
3. **Compile `S_A` with `A_T`** → produce `A_T1`. Because `A_T` was produced by a clean compiler from `S_A`, and `A_T1` was produced by `A_T` from `S_A`, `A_T1` is the output `S_A` would have had if compiled by itself with no Thompson payload.
4. **Compare `A₁` bit-for-bit with `A_T1`**. If they match, `A` is not carrying a self-reinserting backdoor. If they differ, `A` is compromised or your build is non-deterministic (see below).

### Why the second compiler doesn't need to be "trusted"

The magic of DDC: even if `T` is *also* backdoored — with a *different* payload — the chance that both `A` and `T` produce a bit-identical output for a bit-identical input is vanishingly small, because the payloads have to collide exactly. Any independent backdoor produces a non-matching binary and you detect tampering. You need *diversity*, not *trust*.

### The hard part: reproducible builds

DDC requires that **recompiling the same source with the same compiler gives the same bytes**. This is violated by default in almost every build system:

- `__DATE__` / `__TIME__` macros embed the current time.
- Path names leak into debug sections and error messages.
- Build user name, hostname, and environment variables appear in binaries.
- Parallel make can produce different link order → different symbol tables.
- Go embeds the build ID, derived from the module path.

The **Reproducible Builds project** (reproducible-builds.org) tracks and fixes these. Before DDC is possible at all, you need `SOURCE_DATE_EPOCH`, stripped paths, deterministic link order, and a clean build environment. Debian, NixOS, Arch, and Bazel all have mature infrastructure for this.

## The Go bootstrap precaution

Go does not require the compiler to compile itself at any point in its build chain. Instead:

- Go 1.4 is written in C and compiled by a C compiler.
- Go 1.5 is compiled by Go 1.4.
- Go 1.N is compiled by Go 1.(N−1).

There is no point where `Go N → Go N`. The chain bottoms out in C. A Thompson payload in Go N can only survive into Go (N+1) if it can also pattern-match the Go (N+1) compiler source — which, by induction, means it had to be inserted by hand at every release. The payload does not self-replicate across the version bump, so source review *does* work at the bump point.

This was a deliberate decision by Russ Cox and the Go team, explicitly citing Thompson's lecture. See https://go.dev/blog/rebuild for the "Perfectly Reproducible, Verified Go Toolchains" work (2023).

## The practical checklist for supply-chain work

- [ ] Builds are bit-for-bit reproducible given the same source, same toolchain version, same inputs.
- [ ] Two independent builders (different machines, different OSes, different users) produce identical binaries. Publish both.
- [ ] The bootstrap chain does not self-compile. Either (a) bootstrap from C like Go, or (b) cross-compile from a diverse compiler and DDC.
- [ ] The release binary's hash is published by multiple independent parties.
- [ ] All build tools (linker, assembler, libc, libstdc++) are also reproducible — the Thompson attack applies to every stage.
- [ ] CI/CD runners are ephemeral and their images are themselves built reproducibly from a signed base.

## What does NOT defend against the attack

- **"We code-reviewed the compiler source."** The payload isn't in source.
- **"We use a signed binary from the vendor."** Signed by a key held by the vendor — and the vendor's build machine compiled itself.
- **"We scanned the binary with antivirus."** The payload is valid, legitimate-looking machine code for the exact compiler task it performs.
- **"We use an open-source compiler."** Openness of source is orthogonal to trust in the binary.
- **"We rebuild from source every release."** If the rebuild uses the previous release, the payload persists.

## The honest bottom line

For most projects, the Thompson attack is not in your threat model — it requires a persistent, resourceful adversary who has already compromised your toolchain at some point in history. But "not in my threat model" is not the same as "solved." The defense exists (DDC + reproducible builds + bootstrap without self-compilation), it is deployed in Go and Debian, and if you are shipping a compiler, a package manager, a firmware updater, or a signing tool, it **is** in your threat model.

Thompson's closing line in the 1984 lecture: *"You can't trust code that you did not totally create yourself. No amount of source-level verification or scrutiny will protect you from using untrusted code."* The 40-year update: DDC comes close, but only if your builds are reproducible.

## References

- Thompson, K. (1984). *Reflections on Trusting Trust*. CACM 27(8). https://dl.acm.org/doi/10.1145/358198.358210
- Wheeler, D. A. (2005–2009). *Countering Trusting Trust through Diverse Double-Compiling*. https://dwheeler.com/trusting-trust/
- Cox, R. (2023). *Running the "Reflections on Trusting Trust" Compiler*. https://research.swtch.com/nih
- Cox, R. (2023). *Perfectly Reproducible, Verified Go Toolchains*. https://go.dev/blog/rebuild
- Reproducible Builds project. https://reproducible-builds.org
