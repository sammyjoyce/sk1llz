# Portable C Idioms — concrete templates

Read this when: designing a new C API, porting code to a new OS/architecture, or writing code that has to survive the next 20 years of compiler evolution.

---

## `goto cleanup` — the kernel idiom

The canonical shape. Labels are in **reverse acquisition order**. Initialise all resources to "not held" sentinels (NULL, -1) up front so the cleanup path is idempotent.

```c
int do_thing(const char *path, size_t n)
{
    int        ret  = -1;       /* default: failure */
    int        fd   = -1;
    FILE      *fp   = NULL;
    char      *buf  = NULL;
    pthread_mutex_t *lk = NULL;

    fd = open(path, O_RDONLY | O_CLOEXEC);
    if (fd < 0)                     goto out;           /* nothing held yet */

    fp = fdopen(fd, "r");
    if (!fp)                        goto out_fd;
    fd = -1;                        /* fdopen takes ownership of fd */

    buf = malloc(n);
    if (!buf)                       goto out_fp;

    lk = acquire_global_lock();
    if (!lk)                        goto out_buf;

    if (parse(fp, buf, n) < 0)      goto out_lk;

    ret = 0;

out_lk:  release_lock(lk);
out_buf: free(buf);
out_fp:  fclose(fp);                /* closes underlying fd */
out_fd:  if (fd >= 0) close(fd);
out:     return ret;
}
```

**Rules that make this bulletproof:**

1. One `return` point. One success assignment (`ret = 0`) immediately before the cleanup cascade.
2. Every resource has a dedicated label named after the *thing just acquired*.
3. Transfer of ownership (e.g. `fdopen` taking over the fd) is explicit: set the old handle to its sentinel so cleanup doesn't double-free.
4. Never `goto` *forward* past a VLA declaration (`char vla[n];`) or a variably-modified type — C99 6.8.6.1p1 makes that UB.
5. Never `goto` into a block from outside it. Forward jumps are fine only to the cleanup cascade.

## Short read / short write loops

Raw `read(2)` and `write(2)` can return fewer bytes than requested, and can fail with `EINTR` (interrupted by a signal). Any code that doesn't loop is buggy under load.

```c
#include <errno.h>
#include <unistd.h>
#include <sys/types.h>

/* Returns total bytes written, or -1 with errno set.
 * Handles EINTR and partial writes. */
ssize_t write_all(int fd, const void *buf, size_t count)
{
    const char *p = buf;
    size_t      remaining = count;

    while (remaining > 0) {
        ssize_t n = write(fd, p, remaining);
        if (n < 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        if (n == 0) {               /* 0 from write means nothing written;
                                     * treat as error to avoid infinite loop */
            errno = EIO;
            return -1;
        }
        p         += (size_t)n;
        remaining -= (size_t)n;
    }
    return (ssize_t)count;
}

/* Returns bytes read (0 at EOF), or -1 on error.
 * May return less than count ONLY at EOF. */
ssize_t read_all(int fd, void *buf, size_t count)
{
    char  *p = buf;
    size_t remaining = count;

    while (remaining > 0) {
        ssize_t n = read(fd, p, remaining);
        if (n < 0) {
            if (errno == EINTR) continue;
            return -1;
        }
        if (n == 0) break;          /* EOF */
        p         += (size_t)n;
        remaining -= (size_t)n;
    }
    return (ssize_t)(count - remaining);
}
```

**Notes:**

- On non-blocking fds, also handle `EAGAIN` / `EWOULDBLOCK` with `poll`/`epoll`.
- On `SIGPIPE`-generating writes (sockets), install an ignore handler or use `MSG_NOSIGNAL` / `SO_NOSIGPIPE`.
- Always `open(... | O_CLOEXEC)` so fds don't leak into `fork`+`exec`'d children. Without `O_CLOEXEC` there's a race where another thread can `exec` before you `fcntl(FD_CLOEXEC)`.

## Endian-independent integer I/O

Do not `memcpy` into a `uint32_t` and then `htonl` — that's two rules to get right. Byte-assemble directly; the compiler recognises the pattern and emits a single `MOVBE` / `bswap` where available.

```c
#include <stdint.h>

static inline uint32_t load_be32(const uint8_t *p) {
    return ((uint32_t)p[0] << 24) |
           ((uint32_t)p[1] << 16) |
           ((uint32_t)p[2] <<  8) |
           ((uint32_t)p[3]);
}

static inline void store_be32(uint8_t *p, uint32_t v) {
    p[0] = (uint8_t)(v >> 24);
    p[1] = (uint8_t)(v >> 16);
    p[2] = (uint8_t)(v >>  8);
    p[3] = (uint8_t)(v);
}

static inline uint64_t load_le64(const uint8_t *p) {
    return ((uint64_t)p[0])       |
           ((uint64_t)p[1] <<  8) |
           ((uint64_t)p[2] << 16) |
           ((uint64_t)p[3] << 24) |
           ((uint64_t)p[4] << 32) |
           ((uint64_t)p[5] << 40) |
           ((uint64_t)p[6] << 48) |
           ((uint64_t)p[7] << 56);
}
```

**Why this is better than `ntohl`:**

- Works on any alignment (`p` can be anywhere in a byte buffer).
- No strict-aliasing violation.
- Independent of host endianness — the code expresses the *wire* format directly.
- `ntohl` is the wrong tool anyway: it only handles 32-bit, is host-endian-dependent, and requires an aligned load.

## Opaque handle pattern

The canonical way to hide implementation in C. Users see only a forward declaration; the struct definition lives in the `.c` file.

```c
/* widget.h — public API */
#ifndef MYPROJ_WIDGET_H
#define MYPROJ_WIDGET_H

#include <stddef.h>

typedef struct widget widget_t;       /* forward decl — opaque */

widget_t *widget_create(size_t capacity);
void      widget_destroy(widget_t *w);
int       widget_push(widget_t *w, int value);    /* 0 ok, -1 err */
size_t    widget_len(const widget_t *w);

#endif
```

```c
/* widget.c — implementation */
#include "widget.h"
#include <stdlib.h>
#include <string.h>

struct widget {
    int    *data;
    size_t  len;
    size_t  cap;
};

widget_t *widget_create(size_t capacity)
{
    widget_t *w = malloc(sizeof *w);
    if (!w) return NULL;
    w->data = malloc(capacity * sizeof *w->data);
    if (!w->data) { free(w); return NULL; }
    w->len = 0;
    w->cap = capacity;
    return w;
}

void widget_destroy(widget_t *w)
{
    if (!w) return;                    /* free(NULL) is fine,
                                        * but make the contract explicit */
    free(w->data);
    free(w);
}
```

**Why this matters:**

- ABI stability: you can add fields to `struct widget` without recompiling callers.
- Prevents callers from stack-allocating a half-initialised widget.
- Forces construction through the blessed constructor where invariants are established.
- Note: `sizeof(widget_t)` is *unknown* to callers. That's the point.

## Flexible array members (C99 6.7.2.1p18)

For variable-length tail data — far safer than `char data[1]` tricks.

```c
struct packet {
    uint32_t len;
    uint32_t kind;
    uint8_t  payload[];    /* flexible array member — NOT [0], NOT [1] */
};

struct packet *packet_new(uint32_t kind, uint32_t len)
{
    /* sizeof(struct packet) does NOT include the FAM */
    struct packet *p = malloc(sizeof *p + len);
    if (!p) return NULL;
    p->kind = kind;
    p->len  = len;
    return p;
}
```

**Rules:**

- FAM must be the **last** member of the struct.
- Struct must have at least one other member before it.
- `sizeof(struct packet)` gives you the size *without* the FAM (may include padding).
- Cannot be a member of another struct or an array element.
- Never use the old `char data[1]` trick in new code — it's UB to access beyond index 0 per strict reading of the standard; FAM is the standards-blessed form.

## Header design checklist

- [ ] Guard with `#ifndef PROJECT_PATH_FILE_H`, not `#pragma once`.
- [ ] Include *only* what the header needs to compile — never pull in `<stdio.h>` "for the user's convenience."
- [ ] Use forward declarations (`struct foo;`) instead of including another header where possible.
- [ ] Declare every function as `extern` implicitly (never `static` in a header — that creates one copy per TU).
- [ ] Mark read-only parameters `const` (pointer-to-const, not const-pointer unless you mean it).
- [ ] Document ownership in a one-line comment per function: *"Caller must free the returned pointer with `foo_free`."*
- [ ] Document thread safety explicitly: *"Not thread-safe; caller must serialise."*
- [ ] For C libraries callable from C++: wrap in `#ifdef __cplusplus extern "C" { ... } #endif`.
- [ ] Never define types in more than one header (one definition rule — violated, you get ODR-style UB in linking).
- [ ] Use `size_t` for sizes/counts, `ssize_t` for signed counts, `ptrdiff_t` for pointer differences, `intptr_t`/`uintptr_t` for pointer-to-integer round trips.

## Safe string copy helper (replacement for `strncpy`)

```c
/* Copies at most dstsize-1 bytes, always null-terminates if dstsize > 0.
 * Returns length of src (so callers can detect truncation with
 * rv >= dstsize). Same semantics as OpenBSD strlcpy. */
size_t str_lcpy(char *dst, const char *src, size_t dstsize)
{
    size_t srclen = strlen(src);
    if (dstsize != 0) {
        size_t copylen = (srclen >= dstsize) ? dstsize - 1 : srclen;
        memcpy(dst, src, copylen);
        dst[copylen] = '\0';
    }
    return srclen;
}
```

glibc refuses to ship `strlcpy`; musl and BSD libc have it. Keep a private copy or use `snprintf(dst, size, "%s", src)` which is standards-guaranteed portable but slower.

## Two's-complement assumption

C23 finally mandates two's complement for signed integer representation. Before C23, sign-magnitude and ones-complement were permitted in theory. In practice no mainstream hardware has used anything else since the 1970s. If you need to lock this down pre-C23:

```c
_Static_assert((-1 & 3) == 3, "signed integers must be two's complement");
_Static_assert((int)0x80000000 == INT_MIN, "32-bit int must exist");
```

These fire at compile time on the one-in-a-million platform that would break your bit-twiddling code.
