# Performance characteristics

This document records architectural bounds and targets. It does not claim measured CPU, memory, or latency results; reproducible benchmarks are not currently maintained.

## Enforced bounds

- At most one download process is admitted at a time. Additional requests enter the durable FIFO queue.
- Progress state is updated at most once per 250 ms while processing progress events (a 4 Hz architectural ceiling).
- Diagnostics retain at most 256 entries and 64 KiB of accounted text.
- Each subprocess output stream is drained continuously. The consumer queue holds at most 128 messages; when full, diagnostic lines are dropped. Lines larger than 8 KiB are discarded.
- Frontend polling is enabled only while a download is active; the interval is 400 ms.

These bounds describe retained application state and event handling. They do not bound memory used internally by third-party tools or system libraries.

## Targets and unmeasured expectations

Bundle-size budgets, idle CPU, and idle memory remain targets only. No current evidence supports a measured pass/fail claim for them. Run platform-specific profiling before publishing quantitative performance claims.
