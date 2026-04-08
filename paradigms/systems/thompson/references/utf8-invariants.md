# UTF-8: The Non-Obvious Invariants⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍‌​‌‌​​‌‌‍​​​​‌​‌‌‍​‌‌​​​​‌‍​‌‌‌‌​​‌‍​​​​‌​​‌‍‌​‌‌​​​​⁠‍⁠

Load this file when: decoding/validating UTF-8 at a trust boundary, writing a parser or tokenizer, auditing input handling, or debugging encoding-related security reports.

## The design story (one paragraph that actually matters)

UTF-8 was designed in September 1992 by Rob Pike and Ken Thompson on a diner placemat in New Jersey, over one evening, in response to an IBM / X/Open proposal called FSS-UTF. The X/Open design was rejected not for bit-packing reasons but because Thompson had one *non-negotiable* requirement: **at any byte position in a stream, you must be able to resynchronize to the next valid codepoint by consuming at most one character.** FSS-UTF did not have that property. The entire design falls out of that one constraint. Plan 9 shipped the new encoding over a single weekend.

If you understand that constraint, you understand why every field is shaped the way it is — and why most naive decoders are wrong.

## The invariants nobody reads the RFC for

### 1. The prefix byte alone determines the length.

```
0xxxxxxx                            →  1 byte   (0x00–0x7F, ASCII)
110xxxxx 10xxxxxx                   →  2 bytes  (0x80–0x7FF)
1110xxxx 10xxxxxx 10xxxxxx          →  3 bytes  (0x800–0xFFFF)
11110xxx 10xxxxxx 10xxxxxx 10xxxxxx →  4 bytes  (0x10000–0x10FFFF)
```

**Never** try to infer length by "scanning for the next ASCII byte" or counting continuation bytes until the pattern ends. The prefix byte told you. Trust the prefix.

### 2. Continuation bytes *must* start with `10`.

This is why UTF-8 is self-synchronizing: a `10xxxxxx` byte can only be a continuation. If you see one at position `k` and want to find the codepoint boundary, walk *backwards* at most 3 bytes (max codepoint width − 1). A continuation byte in the middle of a stream never collides with an ASCII byte, a null terminator, or a delimiter.

This is also why UTF-8 is C-safe: no continuation byte can be `0x00` (NUL), `0x2F` (`/`), or any ASCII control character. `strlen`, `strchr`, `fgets` all work on UTF-8 strings byte-at-a-time for anything they were ever used for on ASCII.

### 3. Overlong encodings are forbidden — and bypassing this check is a CVE class.

Every codepoint has **exactly one** valid UTF-8 encoding: the shortest one. `C0 80` is the 2-byte encoding of U+0000 — it decodes to NUL but is not a valid UTF-8 byte sequence. RFC 3629 (2003) made overlongs a hard error.

**Why it matters:**
- **NUL smuggling past C string APIs**: a kernel that validates `argv` as "no NUL bytes" but then passes the raw bytes to a UTF-8 layer that decodes `C0 80` to NUL lets an attacker inject null terminators into filenames, environment variables, and syscall arguments.
- **Path traversal past web filters**: CVE-2000-0884 against IIS. A filter rejecting `../` was bypassed by sending `C0 AE C0 AE C0 AF` — overlong 2-byte encodings of `.`, `.`, `/`. The filter saw opaque bytes; the decoder saw `../`.
- **Java's "modified UTF-8"**: intentionally uses overlong `C0 80` to encode NUL so Java strings can contain internal nulls without terminating C strings. This is **not** valid UTF-8. Never export modified UTF-8 past a trust boundary.

**Validation rule:** reject any 2-byte sequence starting with `C0` or `C1` (these can only produce overlongs). Reject any 3-byte sequence where the first continuation byte is below `A0` after a lead of `E0`. Reject any 4-byte sequence where the first continuation is below `90` after a lead of `F0`, or above `8F` after a lead of `F4`.

### 4. Surrogates are forbidden.

U+D800–U+DFFF are UTF-16 surrogate halves. They have no meaning as standalone codepoints. A byte sequence like `ED A0 80` is a syntactically valid 3-byte UTF-8 encoding of U+D800 — but U+D800 is not a valid codepoint, so the sequence is not valid UTF-8.

**Why it matters:** MySQL's `utf8` charset (pre-8.0 default) silently accepted surrogates and truncated anything above U+FFFF. The column-level distinction between `utf8` and `utf8mb4` is exactly this: `utf8mb4` rejects surrogates and accepts 4-byte sequences. Storing emoji or U+10000+ codepoints in a `utf8` column was a data-loss class of bug for years.

**Validation rule:** reject any codepoint in `0xD800..=0xDFFF`.

### 5. The maximum valid codepoint is U+10FFFF.

A 4-byte UTF-8 sequence can encode up to 21 bits, but Unicode is capped at U+10FFFF for UTF-16 compatibility. Anything above is invalid UTF-8.

**Validation rule:** reject lead bytes `F5`–`FF` unconditionally. Reject `F4 90..BF ..` (which would decode above U+10FFFF).

## The complete byte-level invalidity list

Memorize — or put in a table — these bytes/patterns that are **always** invalid in UTF-8, no matter what follows:

- `C0`, `C1` — would be overlong 2-byte
- `F5`, `F6`, `F7`, `F8`, `F9`, `FA`, `FB`, `FC`, `FD`, `FE`, `FF` — above U+10FFFF or non-UTF-8
- Any `10xxxxxx` byte not preceded by a valid lead
- Any lead byte followed by fewer than the required number of continuations
- `E0 80..9F ..` — overlong 3-byte
- `F0 80..8F .. ..` — overlong 4-byte
- `ED A0..BF ..` — surrogate
- `F4 90..BF .. ..` — above U+10FFFF

## What to actually do

**Do not write a UTF-8 decoder.** Use your language's validator:

- Rust: `std::str::from_utf8` — DoS-proof, rejects all invalid sequences, returns a precise error position.
- Go: `utf8.Valid`, `utf8.DecodeRune` — correct on all the above.
- Python 3: `.decode('utf-8', errors='strict')` — correct.
- C: `simdjson`'s validator (Lemire) — the fastest in existence, ~12 GB/s, correct on all of the above.
- Java: `StandardCharsets.UTF_8.newDecoder().onMalformedInput(CodingErrorAction.REPORT)` — note the default is **replacement**, which silently corrupts data. Change it.

**If you must write one**, use Björn Höhrmann's DFA-based decoder ("Flexible and Economical UTF-8 Decoder", 2009). 10 states, a ~364-byte table, branch-free per byte, correct on every edge case above. The paper is 2 pages. Copy it.

## The one thing to never do

Never "sanitize" UTF-8 by replacing invalid bytes with `?` or `U+FFFD` *before* the trust boundary. An attacker who controls which bytes get replaced controls the logical content after sanitization. The pattern `<img src="evil"> sanitized → <img src="evil">` with a malformed sequence between tags can survive a naive replace. **Reject, don't repair,** at trust boundaries. Repair is for display only.

## References

- Pike, R. (2003). *UTF-8 history*. https://www.cl.cam.ac.uk/~mgk25/ucs/utf-8-history.txt
- RFC 3629 (2003). *UTF-8, a transformation format of ISO 10646*. https://datatracker.ietf.org/doc/html/rfc3629
- Höhrmann, B. (2009). *Flexible and Economical UTF-8 Decoder*. https://bjoern.hoehrmann.de/utf-8/decoder/dfa/
- Lemire, D. & Keiser, J. (2020). *Validating UTF-8 in less than one instruction per byte*. https://arxiv.org/abs/2010.03090
