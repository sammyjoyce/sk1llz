# WAL and durability: what goes wrong when you weren't looking⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​‌​‌​​​​‍‌​​​‌‌​‌‍​‌‌​‌‌​‌‍‌‌​​​‌​‌‍​​​​‌​‌​‍​​‌​​‌‌‌⁠‍⁠

Sources: Gray & Reuter, *Transaction Processing: Concepts and Techniques* (1993); fsync-gate discussions (2018); PostgreSQL and RocksDB design docs.

## The WAL protocol (one-line version)

**Write the log before the data, flush the log before the ACK, never overwrite a log record.** If you can articulate all three parts and enforce all three parts, you have 90% of durability correct.

## The fsync gate: why calling fsync() is not enough

In 2018 it was discovered that PostgreSQL — and nearly every other major database — handled `fsync()` errors incorrectly on Linux. The mechanics:

1. You `write()` a dirty page; it sits in the kernel page cache.
2. The kernel eventually tries to write it back to the disk.
3. The write fails (bad sector, USB yanked, storage target gone).
4. The kernel marks the page **clean** (dirty bit cleared) and records the error on the inode.
5. The next caller of `fsync()` on that fd receives the error — **once**.
6. A subsequent `fsync()` returns success, because there are no dirty pages on that inode anymore.
7. Your database thinks the data is durable. It is not. It is gone.

**The safe pattern:** on any `fsync()` error, treat the database as corrupt and **panic the process**. Do not retry. Let WAL recovery on restart re-derive state from the log. This is what PostgreSQL now does (the `data_sync_retry` knob exists but defaults to `panic`).

**Additional traps:**
- On per-fd error tracking, a process that `open()`s after an error will *not* see the prior error at all. Your durability is at the mercy of which process saw which error first.
- `fdatasync()` has the same problem.
- XFS and ext4 behave slightly differently; BSDs differently again. Never assume fsync semantics; test them.

## The hidden write-back caches

`fsync()` only flushes the kernel page cache to the block device. Past that, there are layers you do not control:

| Layer                  | Has a write-back cache by default? |
|------------------------|------------------------------------|
| Kernel page cache      | Yes (flushed by fsync) |
| Block device (disk cache)   | **Yes** on consumer SSDs/HDDs |
| RAID controller cache  | Usually yes (BBU/NV required for safety) |
| Virtualization layer (cloud disks) | Varies — read the provider docs |
| NAND SSD internal buffer | Yes — may need `nvme sync` or FUA writes |

**Test reality, not documentation:** pull the power mid-commit, count lost transactions, iterate. Gray's rule: "If you have not tested your recovery path with a power-pull, you do not have a recovery path."

Turn off consumer-drive write caches with `hdparm -W0` (SATA) or disable volatile writes on NVMe. On RAID controllers, require a BBU or supercap. On cloud disks, read the specific fsync guarantees — some providers honor them, some silently don't.

## Torn writes and the WAL/data asymmetry

Databases typically use 8 KB pages. Filesystems and disks use 4 KB sectors. A crash during an 8 KB write can leave sector 0 written and sector 1 not — a **torn write**.

- **WAL survives torn writes naturally.** Every WAL record has a checksum. On recovery, you scan forward until a checksum fails, then truncate. The last half-written record is discarded. Because the transaction was not yet ACKed, the client never saw it "commit."
- **Data files do NOT survive torn writes naturally.** An overwritten page with a bad checksum can't be reconstructed from the WAL alone — the WAL only records deltas from a prior valid state.

The standard mitigation is **Full Page Writes (FPW)**: the first time a page is dirtied after a checkpoint, write the entire page to the WAL, not just the delta. This lets recovery re-derive the whole page. Postgres's `full_page_writes = on` (default) enables this and is why turning it off for "performance" is usually a disastrous trade.

Alternative: use a filesystem/device that guarantees atomic 8 KB writes (ZFS; some enterprise SSDs with atomic write support; InnoDB's double write buffer).

## Group commit: the 10–100× throughput knob

A single `fsync()` on a modern NVMe drive takes ~50 µs; on a spinning disk, ~5 ms; on an EBS volume, ~1 ms. If you fsync per transaction, your commit throughput is `1 / fsync_time` regardless of how much CPU you have.

**Group commit** batches pending commits, fsyncs once, then ACKs them all. The algorithm:

1. Each writer appends its WAL records to an in-memory buffer, then blocks on a completion semaphore.
2. A single "group committer" thread (or the first writer to notice) flushes the buffer, calls `fsync()`, then wakes all waiting writers.
3. Optionally, the committer delays 1–10 ms to collect a larger batch. More writers ⇒ higher throughput, marginally higher per-transaction latency.

**Tuning:**
- No delay: latency ≈ `fsync_time`; throughput ≈ `1 / fsync_time` (unchanged).
- 1 ms delay with 10 concurrent writers: latency ≈ `1 ms + fsync_time`; throughput ≈ `10 / fsync_time` (10×).
- The batch size is self-limiting — more load = larger batches = more amortization, up to the WAL buffer size.

PostgreSQL does this automatically via `commit_delay` and `commit_siblings`. RocksDB has `WriteBatch` and built-in group commit. If you're building from scratch, you need this from day one; bolting it on later requires reworking the commit path.

## Checkpointing: the other knob that controls recovery time

The WAL grows unboundedly without checkpoints. A checkpoint:
1. Flushes all dirty pages to the data files.
2. Writes a CHECKPOINT record to the WAL.
3. Truncates (or archives) WAL segments older than the checkpoint.

**Recovery time is bounded by the WAL length since the last checkpoint**, plus the time to redo it. Gray's rule of thumb (from the Five-Minute Rule paper) is a 5-minute checkpoint interval. Too frequent ⇒ I/O thrash. Too rare ⇒ unbounded recovery time and oversized WAL.

**The paradox:** checkpointing causes I/O spikes that can stall user transactions. Techniques to smooth this out:
- Fuzzy checkpoints — spread dirty page flushes across the interval.
- Background writer that trickles dirty pages out continuously.
- Sized-based triggers (WAL grew by X MB) in addition to time-based.

PostgreSQL `checkpoint_completion_target` controls the spread; tune to 0.7–0.9 to smooth the I/O.

## Synchronous commit levels: the spectrum

Not every write needs the same guarantee. PostgreSQL's `synchronous_commit` taxonomy (applies conceptually to any system):

| Level         | Guarantee on client ACK |
|---------------|--------------------------|
| `off`         | In the WAL buffer only. Loss window: WAL buffer flush interval (~200 ms). |
| `local`       | WAL fsynced to local disk. Loss window: none locally; all replicas. |
| `remote_write`| WAL received by the sync replica's buffer. Loss window: replica crash before fsync. |
| `on` (default)| WAL fsynced on the primary (and on sync replicas, if configured). |
| `remote_apply`| WAL applied (visible to readers) on sync replicas. Strongest; highest latency. |

**The rule:** mix levels per-transaction where it matters. Ledger writes = `remote_apply`. Audit logs = `local`. Analytics bulk loads = `off`.

## Transaction states and the commit point

From Gray/Reuter, the canonical transaction state machine:

    ACTIVE ──► PARTIALLY_COMMITTED ──► COMMITTED
       │                                 ▲
       ▼                                 │
     FAILED ────────────► ABORTED ◄──────┘  (only if the commit record is not yet durable)

The **commit point** is the instant the commit record becomes durable in the WAL. Before that point, the transaction can be aborted. After that point, the transaction *must* eventually be applied — even if the system crashes and restarts. There is no third option. If your code has ambiguity about "was this committed?" at any specific line, you have a bug.

## The fast-path checklist

Before shipping any durability-critical code, walk the list:

- [ ] Every commit path ends in an `fsync()` (or equivalent) before the ACK.
- [ ] Every `fsync()` error is fatal — the process panics or the DB goes read-only.
- [ ] WAL records have checksums; recovery truncates at the first bad one.
- [ ] Data pages are protected against torn writes (FPW, atomic writes, or double-write buffer).
- [ ] Write-back caches below the filesystem are either disabled or BBU/supercap-backed.
- [ ] Group commit is implemented *and* measured — you know your batch size under load.
- [ ] Checkpoints are triggered by both time and WAL size; interval bounds recovery time.
- [ ] You have run a power-pull test on the real hardware and counted lost transactions.
- [ ] Recovery is *replayed*, not reconstructed from backups.
