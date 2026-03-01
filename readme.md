# Distributed Background Job Processor (Rust)

A fault-tolerant, PostgreSQL-backed background job processing system built in Rust using Tokio and Axum.

Designed with infrastructure concerns in mind: concurrency control, crash recovery, lease-based ownership, retry backoff strategies, and horizontal scalability.

---

## Design Goals

* Deterministic job state transitions
* Safe concurrent job claiming across processes
* Crash recovery without manual intervention
* Pluggable retry strategies
* Horizontal scalability without distributed locks
* Clean failure boundaries and structured error handling

---

## System Overview

The system consists of:

* **HTTP API Layer** (Axum)
* **Async Worker Runtime** (Tokio)
* **PostgreSQL Coordination Layer**
* **Retry Policy Abstraction**
* **Lease-Based Ownership Model**

Workers coordinate exclusively through PostgreSQL row-level locking (`FOR UPDATE SKIP LOCKED`). No in-memory shared state is required for correctness.

---

## Core Concepts

### Job Model

Each job contains:

* `id`
* `job_type`
* `payload` (JSONB)
* `state`
* `attempts`
* `max_attempts`
* `run_at`
* `locked_by`
* `lease_expires_at`
* `timestamps`

States:

```text
queued
running
succeeded
failed
dead
canceled
```

---

## Concurrency Model

### Claim Path

Workers claim jobs using:

```sql
SELECT id
FROM jobs
WHERE state = 'queued'
  AND run_at <= NOW()
ORDER BY run_at
FOR UPDATE SKIP LOCKED
LIMIT 1;
```

This ensures:

* At-most-one worker claims a job at a time
* No blocking between workers
* No centralized mutex bottleneck
* Safe multi-process coordination

### Execution Path

1. Begin transaction
2. Claim job row
3. Update state → `running`
4. Set lease expiration
5. Commit transaction
6. Execute job outside transaction
7. Update state on completion

---

## Leasing

Leasing prevents permanently stuck jobs.

On claim:

* `state = running`
* `locked_by = worker_id`
* `lease_expires_at = now + lease_duration`

If a worker crashes:

* Lease expires
* Another worker reclaims job
* No manual intervention required

Lease recovery query:

```sql
UPDATE jobs
SET state = 'queued',
    locked_by = NULL,
    lease_expires_at = NULL
WHERE state = 'running'
  AND lease_expires_at < NOW();
```

Leasing separates **claim-time safety** from **execution-time ownership**.

---

## Retry Policies

Retry behavior is abstracted behind a strategy trait:

```rust
pub trait RetryPolicy {
    fn next_delay(&self, attempts: u32) -> Duration;
}
```

Implemented strategies:

* Exponential backoff (with jitter)
* Linear backoff

Exponential backoff uses jitter to mitigate synchronized retry spikes.

Retry delay is capped to prevent unbounded scheduling drift.

---

## Dead Letter Queue

If `attempts >= max_attempts`:

* Job transitions to `dead`
* Available via API inspection endpoint
* No automatic rescheduling

Dead jobs remain persisted for operational analysis.

---

## Cancellation

Cancelable states:

* `queued`
* `running` (best-effort)

Cancellation enforces state validation to prevent illegal transitions.

---

## Graceful Shutdown

On termination signal:

* Stop claiming new jobs
* Allow in-flight executions to finish
* Release leases if needed
* Exit cleanly

Prevents partial state transitions or orphaned claims.

---

## Metrics

Exposes runtime counters:

* `jobs_processed`
* `jobs_failed`
* `jobs_dead`
* `avg_execution_latency`

Metrics are maintained via atomic counters and exposed through `/metrics`.

---

## Database Schema (Simplified)

```sql
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    job_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    state TEXT NOT NULL,
    attempts INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL,
    run_at TIMESTAMP NOT NULL,
    locked_by TEXT,
    lease_expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_jobs_state_run_at
ON jobs (state, run_at);
```

---

## Scaling Characteristics

* Supports configurable worker count per instance
* Safe multi-instance deployment
* No shared memory coordination required
* Concurrency control delegated to PostgreSQL MVCC

Throughput scales until bounded by database IO or CPU limits.

---

## Failure Model

Handled scenarios:

* Worker crash during execution
* Lease expiration
* Retry exhaustion
* Duplicate concurrent workers
* Graceful termination

Not addressed:

* External side-effect idempotency (delegated to downstream systems)
* Cross-database distributed transactions

---

## Tech Stack

* Rust (stable)
* Tokio
* Axum
* sqlx
* PostgreSQL
* thiserror
* Docker

---

## Running

```bash
docker-compose up --build
```

Service exposed at:

```
http://localhost:8484
```

---
