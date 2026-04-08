# Code patterns: WAL, transaction manager, idempotent executor

Reference implementations for the patterns described in the main skill. These are skeletons to illustrate shape and invariants, not production code. Production code must add checksums, error handling, concurrency safety, and failure injection tests.

## Pattern 1: Write-Ahead Log (append, fsync, checkpoint)

```python
import os
import struct
import zlib
from dataclasses import dataclass
from enum import Enum
from typing import Any, Iterator

class LogRecordType(Enum):
    BEGIN      = 1
    UPDATE     = 2
    COMMIT     = 3
    ABORT      = 4
    CHECKPOINT = 5

@dataclass
class LogRecord:
    lsn: int              # Log Sequence Number
    txn_id: int
    record_type: LogRecordType
    table: str = ""
    key: Any = None
    before_value: Any = None  # for UNDO
    after_value: Any = None   # for REDO

class WriteAheadLog:
    """
    Gray's WAL protocol:
      1. Write the log record BEFORE the data page.
      2. fsync before ACKing the commit.
      3. Checksum every record; truncate recovery at the first bad checksum.
      4. On fsync error, PANIC — do not retry (see wal-durability.md).
    """

    def __init__(self, path: str):
        self.path = path
        self.fd = os.open(path, os.O_RDWR | os.O_CREAT | os.O_APPEND, 0o600)
        self.lsn = self._recover_last_lsn()

    def append(self, record: LogRecord) -> int:
        self.lsn += 1
        record.lsn = self.lsn
        payload = self._serialize(record)
        checksum = zlib.crc32(payload)
        frame = struct.pack(">II", len(payload), checksum) + payload
        os.write(self.fd, frame)
        try:
            os.fsync(self.fd)
        except OSError as e:
            # Gray's rule + fsync-gate: EIO from fsync is fatal.
            # Do NOT retry. Do NOT continue. Panic the process so WAL recovery
            # on restart re-derives state from the log.
            os._exit(70)  # EX_SOFTWARE
        return self.lsn

    def iter_from(self, start_lsn: int) -> Iterator[LogRecord]:
        """Scan forward; stop at the first record with a bad checksum (torn write)."""
        with open(self.path, "rb") as f:
            while True:
                header = f.read(8)
                if len(header) < 8:
                    return
                length, expected_crc = struct.unpack(">II", header)
                payload = f.read(length)
                if len(payload) < length:
                    return  # torn write; truncate here
                if zlib.crc32(payload) != expected_crc:
                    return  # corruption or torn write; truncate here
                record = self._deserialize(payload)
                if record.lsn >= start_lsn:
                    yield record

    def _recover_last_lsn(self) -> int:
        last = 0
        for r in self.iter_from(0):
            last = r.lsn
        return last

    def _serialize(self, record: LogRecord) -> bytes: ...
    def _deserialize(self, payload: bytes) -> LogRecord: ...
```

**What this illustrates:**
- Every record is framed with `(length, crc32, payload)`.
- `fsync()` errors are fatal — no retry, no continuation.
- Recovery scans until the first checksum failure and truncates there. Torn writes are survivable because the partial record is discarded.
- `lsn` is assigned in-order, monotonic.

**What this skeleton omits (and your production code needs):**
- Group commit: batch multiple records, fsync once, wake all waiters.
- File rotation and checkpoint-triggered truncation.
- Locking around `append()` for concurrent writers.
- A persistent "known-good LSN" cursor.

## Pattern 2: Transaction manager with lifecycle guarantees

```python
from contextlib import contextmanager
from threading import Lock
from typing import Generator

class Transaction:
    def __init__(self, txn_id: int, wal: WriteAheadLog, storage: "Storage"):
        self.txn_id = txn_id
        self.wal = wal
        self.storage = storage
        self.undo_records: list[LogRecord] = []
        self.locks_held: set = set()

    def update(self, table: str, key: Any, new_value: Any) -> None:
        old_value = self.storage.read(table, key)
        rec = LogRecord(
            lsn=0, txn_id=self.txn_id, record_type=LogRecordType.UPDATE,
            table=table, key=key, before_value=old_value, after_value=new_value,
        )
        self.wal.append(rec)          # log BEFORE touching the data page
        self.storage.write(table, key, new_value)
        self.undo_records.append(rec)

    def rollback(self) -> None:
        # Apply before-images in reverse order
        for rec in reversed(self.undo_records):
            self.storage.write(rec.table, rec.key, rec.before_value)

    def release_locks(self) -> None: ...

class TransactionManager:
    """Enforces ACID via WAL + locking. Aborts on exception; commits on success."""

    def __init__(self, wal: WriteAheadLog, storage: "Storage"):
        self.wal = wal
        self.storage = storage
        self.active: dict[int, Transaction] = {}
        self.lock = Lock()
        self.next_id = 0

    @contextmanager
    def transaction(self) -> Generator[Transaction, None, None]:
        txn = self._begin()
        try:
            yield txn
            self._commit(txn)
        except Exception:
            self._abort(txn)
            raise

    def _begin(self) -> Transaction:
        with self.lock:
            tid = self.next_id
            self.next_id += 1
        self.wal.append(LogRecord(lsn=0, txn_id=tid, record_type=LogRecordType.BEGIN))
        txn = Transaction(tid, self.wal, self.storage)
        self.active[tid] = txn
        return txn

    def _commit(self, txn: Transaction) -> None:
        # The commit record is the "commit point": before fsync, the txn can abort.
        # After fsync returns, the txn MUST eventually be applied.
        self.wal.append(LogRecord(lsn=0, txn_id=txn.txn_id, record_type=LogRecordType.COMMIT))
        txn.release_locks()
        del self.active[txn.txn_id]

    def _abort(self, txn: Transaction) -> None:
        txn.rollback()
        self.wal.append(LogRecord(lsn=0, txn_id=txn.txn_id, record_type=LogRecordType.ABORT))
        txn.release_locks()
        del self.active[txn.txn_id]
```

**What this illustrates:**
- The commit record is the commit point — the single instruction that matters.
- Undo records are held in memory and applied in reverse on abort.
- Exceptions flow through the context manager → abort → re-raise.
- Locks are released *after* the commit/abort record is durable.

**Recovery logic (not shown):** on restart, scan the WAL from the last checkpoint. For every txn_id with a COMMIT record, REDO the UPDATEs. For every txn_id without a COMMIT or ABORT record (incomplete transactions), UNDO the UPDATEs using before-images. This is the ARIES-lite structure.

## Pattern 3: Idempotent executor (client-generated keys)

```python
from datetime import datetime, timedelta
from typing import Any, Callable

class IdempotentExecutor:
    """
    Turns at-least-once into exactly-once *at the application layer*.
    The key is generated by the CLIENT, before the first attempt, and
    the (key, result) is persisted INSIDE the same transaction as the work.
    """

    def __init__(self, storage: "KeyValueStore", ttl: timedelta = timedelta(hours=24)):
        self.storage = storage
        self.ttl = ttl

    def execute(self, idempotency_key: str, operation: Callable[[], Any]) -> Any:
        # Fast path: previous result exists and is still valid → return it.
        prior = self.storage.get_if_fresh(idempotency_key, self.ttl)
        if prior is not None:
            return prior

        # Slow path: run the operation and atomically persist (key → result).
        # The storage.put_if_absent must be part of the same transaction as the work,
        # otherwise you have a race window where two retries both execute and only
        # one persists the key — classic double-charge bug.
        result = operation()
        self.storage.put_if_absent(idempotency_key, result, self.ttl)
        return result
```

**Critical invariants:**
1. **Key is client-generated.** If the server generates the key, a timeout leaves the client unable to retry safely.
2. **Key and result are persisted in the same transaction as the work.** A separate write after the work creates a race: if the process crashes between the work and the key-write, a retry reruns the work.
3. **TTL is longer than any realistic retry window.** 24 h is the default; ledgers use 7+ days. Shorter TTLs can lose the idempotency guarantee if a client retries after the TTL expires.
4. **The key index is unique.** If you're using Postgres, make it a `UNIQUE` constraint and handle the duplicate-key error as a hit on the fast path.

## Pattern 4: Group commit (pseudocode)

```python
import threading
import time

class GroupCommitter:
    """
    Batches multiple pending commits, fsyncs once, wakes all waiters.
    Latency floor is the fsync time; throughput scales with batch size.
    """

    def __init__(self, wal: WriteAheadLog, batch_window_ms: float = 2.0):
        self.wal = wal
        self.window = batch_window_ms / 1000.0
        self.pending: list[tuple[LogRecord, threading.Event]] = []
        self.lock = threading.Lock()
        self.committer_running = False

    def commit(self, record: LogRecord) -> None:
        event = threading.Event()
        with self.lock:
            self.pending.append((record, event))
            if not self.committer_running:
                self.committer_running = True
                threading.Thread(target=self._drain, daemon=True).start()
        event.wait()  # block until the group fsync completes

    def _drain(self) -> None:
        time.sleep(self.window)  # collect more commits
        with self.lock:
            batch = self.pending
            self.pending = []
            self.committer_running = False
        for rec, _ in batch:
            self.wal.append_no_fsync(rec)  # append all, fsync once
        try:
            self.wal.fsync()
        except OSError:
            os._exit(70)  # fsync-gate: panic on error
        for _, event in batch:
            event.set()  # wake the waiters
```

**Tuning notes:**
- **`batch_window_ms`** is the per-transaction latency tax. 1–10 ms is typical.
- Under light load, you pay the full window for 1 commit — use a minimum batch size threshold to skip the window when there's only one waiter.
- Under heavy load, batches become larger naturally, and per-commit amortized cost approaches `fsync_time / batch_size`.
- **Do not let the batch grow unbounded.** Cap it at the WAL buffer size; flush early if the cap is hit.

## Invariants that apply to every pattern above

1. **Log before data.** Every pattern above writes the WAL record before any persistent side effect.
2. **fsync before ACK.** The client is never told "committed" until the commit record is durable.
3. **Crash on fsync error.** Ever retrying an fsync error is a correctness bug.
4. **Checksums everywhere.** Every persistent record carries a checksum; recovery discards bad records.
5. **Idempotent application.** Every mutation, on both the forward and the undo path, must be idempotent. Recovery may replay the same record multiple times if the restart sequence is interrupted.

If any pattern you write violates one of these five, you have a bug. They are not guidelines.
