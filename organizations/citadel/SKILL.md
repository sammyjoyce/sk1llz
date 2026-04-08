---
name: citadel-low-latency-systems
description: Engineer trading and market-data paths for tail-latency rather than benchmark vanity metrics. Use when requests mention sub-microsecond or jitter-sensitive systems, market making, order gateways, feed handlers, DPDK, Onload, AF_XDP, busy polling, NUMA pinning, interrupt coalescing, or NIC/CPU topology.
tags: trading, low-latency, market-making, hft, dpdk, onload, af-xdp, numa, networking, real-time
---

# Citadel Low Latency Systems⁠‍⁠​‌​‌​​‌‌‍​‌​​‌​‌‌‍​​‌‌​​​‌‍​‌​​‌‌​​‍​​​​​​​‌‍‌​​‌‌​‌​‍‌​​​​​​​‍‌‌​​‌‌‌‌‍‌‌​​​‌​​‍‌‌‌‌‌‌​‌‍‌‌​‌​​​​‍​‌​‌‌‌‌‌‍​‌​​‌​‌‌‍​‌‌​‌​​‌‍‌‌​‌​‌‌​‍​‌​‌‌‌‌‌‍​​‌‌‌​‌​‍​​​​​​‌​‍‌​​‌‌‌‌‌‍‌‌​​​‌‌‌‍​‌​‌​​‌‌‍​​​​‌​​‌‍‌‌​‌​​‌‌⁠‍⁠

This skill is for systems where `p99.99` matters more than average throughput. Default stance: assume the first regression is topology or coherency, not algorithmic complexity.

## Mandatory Context

- Before changing NIC queues, coalescing, or busy polling, read the vendor NIC tuning guide and Linux NAPI busy-poll docs for that exact stack.
- Before changing C-state or isolation policy, read the kernel docs for `nohz_full`, `rcu_nocbs`, and PM QoS.
- Before using Solarflare/Onload `latency-best` profiles, read the vendor warning: copy the profile, rename it, and tune the copy; the shipped profile can change between releases.
- Do not load generic HFT explainers, "what is DPDK" material, or textbook lock-free primers for this task. They add almost no decision value.

## Operating Mindset

Before touching code, ask yourself:

- **Budget ownership:** Where does each `100 ns` go: NIC DMA, cache miss, queue handoff, wakeup, serialization, or risk check?
- **Topology first:** Is the NIC, IRQ, polling thread, and memory on the same socket? If not, you are usually benchmarking UPI traffic, not your code.
- **Tail source:** Is the problem constant overhead or rare spikes? Constant overhead wants cache and branch work; spikes want power, firmware, paging, and IRQ analysis.
- **Wait strategy:** Is this path supposed to spin, busy-poll, or sleep? Mixing strategies usually creates bimodal latency.
- **Coherency path:** Which cache line is bouncing between core and core, or device and core? "Lock-free" is irrelevant if producer and consumer indexes share a line.
- **Freedom rule:** For architecture, reason from principles. For production tuning, change one knob per run and hold traffic shape, core placement, and message size constant.

## Decision Tree

1. If `p50` and `p99` are fine but `p99.99` has millisecond spikes, suspect firmware or memory management before rewriting hot code.
   Check SMIs with `hwlat`, THP/compaction, package C-states, thermal throttling, and stray housekeeping threads.
2. If throughput is acceptable but latency explodes under load, suspect batching.
   Shrink burst sizes, coalescing windows, and queue depths before touching parsing logic.
3. If one socket performs well and another is bad, stop.
   Move the NIC, IRQ affinity, thread pinning, and memory allocation to the same socket before any micro-optimization.
4. If Linux sockets are mandatory and socket count is modest, prefer selective busy-poll over full kernel bypass.
   Kernel `busy_read=50` is the documented starting point; for several hundred sockets `busy_poll=100` is reasonable, and beyond that you usually want epoll with NAPI-aware placement.
5. If you already spin in user space, do not also chase zero interrupt moderation blindly.
   Onload's own spinning guidance pairs `EF_POLL_USEC=100000` with `rx-usecs 60 adaptive-rx off` specifically to avoid interrupt floods while user space is already polling.
6. If ideal locality is impossible in production, do not pretend software can hide it.
   Fall back to the simplest stable model: fixed interrupt moderation plus explicit queue affinity. Accept the median hit instead of shipping a fragile half-spin system with worse tails.

## Heuristics That Actually Matter

- DPDK default burst sizes hide MMIO cost, but the first packet waits for the rest of the burst. `burst=1` lowers latency because the TX tail is advanced immediately; `burst=16` is a throughput play.
- Same-socket DDIO locality is not optional. Intel's published example shows local forwarding at about `112 ns` inbound PCIe read latency and `135 ns` write latency, versus `320 ns` and `240 ns` when the forwarding core runs on the wrong socket, with throughput dropping from `21.1` to `17.1 Mpps` and `12.6 GB/s` of wasted UPI traffic.
- Bigger rings are not "safer" by default. They reduce drops, but they also lengthen queueing time and can evict hot DDIO lines. When `rx_dropped` rises, first ask whether CPU is actually saturated; if not, lower interrupt moderation or shorten the burst before inflating ring depth.
- Mempool recycling is a cache policy, not just an allocator detail. Disabling DPDK mempool cache (`--mbcache=0`) raised inbound PCIe write latency from `135 ns` to `178 ns` and reduced throughput in Intel's test because the NIC stopped landing into warm packet-ring cache lines.
- `mlockall()` is necessary but insufficient. You also need prefaulting. Otherwise the first live packet pays the page-fault tax and page-table population cost, which shows up as "random" warmup jitter.
- Transparent Huge Pages are hostile to low-latency hot paths when the working set is fragmented or short-lived. Red Hat documents cases where large allocations normally below `0.8 s` spiked to `90 s`, and separate NUMA-node slowdowns of about `10%`, because THP scanning and compaction kicked in. Use explicit hugepages for pinned, long-lived rings; set THP to `never` on trading hosts unless you have hard evidence it helps.
- `isolcpus` alone is not isolation. `nohz_full` and `rcu_nocbs` are what stop scheduler ticks and RCU callbacks from landing on your trading cores. Also remember `nohz_full` cannot keep the boot CPU isolated, so plan a housekeeping core on purpose.
- `idle=poll` is a trap of last resort. Kernel docs explicitly warn it can overheat the CPU, induce thermal throttling, and even disable Turbo, which can leave tail latency worse than dynticks. Prefer PM QoS via `/dev/cpu_dma_latency` or `intel_idle.max_cstate` limits first. Also remember PM QoS disappears when the file descriptor closes.
- RDTSC is not ordered. If you time tiny code sections with raw `rdtsc`, you are often timing speculation. Use `rdtscp` or fenced `rdtsc`, verify invariant TSC, and do not compare measurements across sockets or VMs until clock behavior is proven.
- Seqlocks are only for rarely-written, pointer-free snapshots. Under heavy reader pressure they can starve or deadlock writers on RT-style scheduling, and any embedded pointer can become invalid mid-read. For market-data snapshots, use fixed-size POD copies with one writer, or RCU-style indirection if pointers are unavoidable.
- False sharing hides inside "embarrassingly parallel" code. Intel's example shows a `512 B` object with `59 cycles` average access latency because `64 B` per-thread elements crossed cacheline boundaries. Align the array base, not just the struct type.
- Busy-poll epoll only works if all FDs in that epoll instance share a NAPI ID. If you scatter flows arbitrarily across workers, the kernel cannot keep the polling loop local. Use `SO_INCOMING_NAPI_ID`, `SO_REUSEPORT`, or BPF flow steering to preserve queue/thread affinity.
- `gro_flush_timeout` is a balancing knob, not a free win. Too large improves batching but adds unloaded latency; too small lets hardware IRQs fight the user-space busy-poll loop. Tune it only while observing both tail latency and CPU interference.

## NEVER Rules

- NEVER optimize average latency before tail latency because trading losses come from the rare queue stall, not the pretty median. Instead instrument `p50/p99/p99.9/p99.99`, drops, IRQ rate, and temperature on every run.
- NEVER move only the thread and forget the memory because remote packet rings turn every DMA completion into coherency traffic. Instead pin thread, IRQ, hugepages, and NIC to the same socket as a unit.
- NEVER copy vendor `latency-best` profiles directly into production because they can be experimental and may change between releases. Instead clone, rename, and benchmark your own pinned version.
- NEVER disable all interrupt moderation just because a benchmark got faster because once a spinning stack starts flood-interrupting, CPU time disappears into IRQ churn. Instead decide whether the stack is interrupt-driven or spin-driven, then tune coalescing for that single model.
- NEVER trust a lock-free queue that shares producer/consumer counters or status bits on the same cache line because coherency traffic recreates the lock you thought you removed. Instead separate hot writer-owned and reader-owned fields by cache line and verify with hardware counters.
- NEVER use THP as a blanket "bigger pages are faster" rule because compaction and NUMA balancing can inject multi-millisecond stalls far away from the allocation site. Instead use explicit hugepages only for preallocated buffers whose lifetime and placement you control.
- NEVER blame application code first when you see isolated millisecond spikes because SMIs run below the kernel and are invisible to normal profilers. Instead reproduce on an idle host and check with `hwlat` off-production.
- NEVER present TSC numbers as nanoseconds without verifying clock invariance and serialization because the result is numerically precise but physically false. Instead report both cycles and the exact conversion assumption.

## Failure Signatures

- Good medians, terrible idle-time tails: C-state exit latency, package power management, or interrupt moderation.
- Good single-core benchmark, bad multi-core run: false sharing, shared LLC thrash, or remote NUMA placement.
- Good synthetic forwarding, bad production TCP: wrong wait strategy, wrong NAPI/flow affinity, or Onload/kernel profile drift.
- Lower drops after a tuning change but worse fills or acks: you probably traded packet loss for queueing latency.
- "Optimization" only helps cold start: you fixed a page-fault or instruction-cache issue, not steady-state latency.
- Busy-poll helps in test but hurts in prod: your flow-to-worker placement is probably breaking NAPI locality; drop back to explicit interrupt affinity until you can steer by queue.

## Output Contract

When applying this skill, produce:

- the latency budget and which hop owns the tail,
- the chosen wait model: `interrupt`, `busy-poll`, or `full bypass`,
- the topology plan: socket, IRQ, queue, and memory placement,
- the single next experiment, its expected failure mode, and the metric that invalidates the hypothesis.
