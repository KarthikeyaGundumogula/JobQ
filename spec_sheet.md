# 📌 Distributed Job System — Engineering Spec & Progress Tracker

## 0. Project Objective

Build a crash-resilient, PostgreSQL-backed background job processor in Rust with:

* Safe concurrent claiming
* Lease-based recovery
* Pluggable retry policies
* Dead-letter handling
* Graceful shutdown
* Horizontal scalability

---

# PHASE 1 — Core Engine (In-Memory Prototype)

### 1.1 Worker Runtime

* [x] Tokio-based async worker loop
* [x] Configurable worker count
* [x] Proper lock scoping (no lock held across await)
* [x] Job state transitions: queued → running → succeeded/failed
* [x] Attempts incremented on claim

---

### 1.2 Scheduling

* [x] `run_at` timestamp support
* [x] Min-heap scheduling
* [x] Stale heap entry handling
* [x] Retry rescheduling pushes new index entry

---

### 1.3 Retry Logic

* [x] `max_attempts`
* [x] Exponential backoff
* [x] Jitter to mitigate thundering herd
* [x] Max delay cap
* [x] Linear retry policy implementation
* [x] RetryPolicy trait abstraction (clean version)

---

### 1.4 API Layer

* [x] POST /jobs
* [x] GET /jobs/{id}
* [x] POST /jobs/{id}/cancel

---

### 1.5 Error Handling

* [x] `thiserror` integration
* [x] `From` conversions for UUID / Serde
* [x] Structured HTTP error mapping (clean responses)
* [x] No unwrap() in request path
* [x] `cargo clippy` clean

---

# PHASE 2 — PostgreSQL Persistence (Critical Infrastructure Phase)

## 2.1 Database Integration

* [x] Add `sqlx`
* [x] Connection pool configuration
* [x] Migrations setup
* [x] Jobs table schema
* [x] Index `(state, run_at)`
* [ ] Index on `lease_expires_at`

---

## 2.2 Job Claiming via DB

* [ ] Implement `FOR UPDATE SKIP LOCKED`
* [ ] Transaction-based claim
* [ ] Update state to `running`
* [ ] Set `lease_expires_at`
* [ ] Commit before execution

---

## 2.3 Retry + Persistence

* [x] Persist `attempts`
* [x] Persist updated `run_at`
* [x] Persist dead state
* [x] Atomic state updates

---

## 2.4 Lease Recovery

* [ ] On startup: reclaim expired leases
* [ ] Periodic expired lease requeue
* [ ] Test crash scenario manually

---

# PHASE 3 — Reliability & Control

## 3.1 Leasing Model

* [ ] Worker ID generation
* [ ] `locked_by` column
* [ ] Lease duration config
* [ ] Reclaim query
* [ ] Prevent double execution

---

## 3.2 Graceful Shutdown

* [ ] Signal handling (SIGTERM)
* [ ] Stop claiming new jobs
* [ ] Finish in-flight tasks
* [ ] Release leases cleanly
* [ ] Shutdown timeout config

---

## 3.3 Dead Letter Queue

* [ ] State transition to `dead`
* [ ] GET /jobs/dead endpoint
* [ ] Optional: retry dead job endpoint

---

## 3.4 Cancel Endpoint

* [x] State-aware cancel validation
* [x] Cancel queued jobs
* [x] Best-effort cancel running jobs
* [x] Prevent illegal transitions

---

# PHASE 4 — Retry Strategy Abstraction

## 4.1 RetryPolicy Trait

* [ ] Define trait
* [ ] ExponentialBackoff struct
* [ ] LinearBackoff struct
* [ ] Inject policy into worker config
* [ ] Remove hardcoded delay logic

---

# PHASE 5 — Metrics

## 5.1 Runtime Counters

* [ ] Atomic processed counter
* [ ] Atomic failed counter
* [ ] Atomic dead counter
* [ ] Track execution latency
* [ ] Compute rolling average

---

## 5.2 Metrics Endpoint

* [ ] GET /metrics
* [ ] Prometheus-compatible text format (optional)
* [ ] No locking bottlenecks

---

# PHASE 6 — Horizontal Scaling Validation

* [ ] Run multiple worker processes locally
* [ ] Confirm no duplicate execution
* [ ] Confirm SKIP LOCKED behavior
* [ ] Load test basic throughput
* [ ] Validate lease expiration works under crash

---

# PHASE 7 — Production Readiness

* [ ] Dockerfile
* [ ] docker-compose (Postgres + app)
* [ ] Environment-based config
* [ ] README finalized
* [ ] Architecture diagram
* [ ] Integration test (enqueue → process → succeed)
* [ ] Chaos test (kill worker mid-execution)

---

# OPTIONAL (Stretch Goals)

* [ ] Idempotency key support
* [ ] Per-key concurrency limits
* [ ] Rate limiting middleware
* [ ] Prometheus exporter
* [ ] Structured logging (tracing)

---

# Engineering Guarantees (Target)

When complete, system should provide:

* At-least-once execution
* No duplicate concurrent execution
* Crash recovery via lease expiration
* Bounded retry attempts
* Horizontal scalability
* Deterministic state transitions

---

# Estimated Timeline (Realistic)

Phase 2 (DB + Leasing): 2–3 weeks
Phase 3 (Reliability polish): 1–2 weeks
Phase 4–5 (Abstraction + Metrics): 1 week
Phase 6–7 (Hardening): 1 week

---
