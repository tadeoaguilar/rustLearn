# 22 · Kubernetes Operators

## Overview

Kubernetes is a collection of control loops. The Deployment controller
watches Deployments and makes ReplicaSets match; the ReplicaSet controller
makes Pods match. An **operator** adds your own loop for your own kind of
object: users declare *what* they want (`kind: App`, `replicas: 3`) and the
operator keeps doing *whatever it takes* to make it so — creating objects,
repairing drift, cleaning up external resources. In Rust, the
[kube-rs](https://kube.rs/) crates provide the client, the CRD derive, and the
controller runtime.

## What You'll Learn

| Exercise | Topic | The point |
|---|---|---|
| 1 | CRDs | `#[derive(CustomResource)]`, schemas the API server enforces, printer columns |
| 2 | Desired state | Pure functions; labels, owner references, a config hash that rolls pods |
| 3 | Planning | Comparing only owned fields — why naive diffs hot-loop |
| 4 | Status | Phases, conditions, `observedGeneration` |
| 5 | Database operator | External resources, finalizers, credentials that survive reconciles |
| 6 | Reconcilers | Idempotent, level-based reconciliation; drift repair; convergence |
| 7 | Retries and metrics | Per-object backoff, Prometheus counters |
| Bonus | A real cluster | kube-rs `Controller`, server-side apply, RBAC |

## Key Concepts

### The reconcile loop

```
           watch events (App, owned Deployments, ...)
                         │
                         ▼
   ┌──────────── reconcile(app) ─────────────┐
   │ read desired (spec) and observed state  │
   │ make ONE step towards desired           │──▶ Ok(requeue after 5 min)
   │ write status                            │──▶ Err → requeue with backoff
   └─────────────────────────────────────────┘
```

Reconcile is **level-based**: it's told *which* object to look at, never
*what happened*. Events can be merged, delayed or lost; the operator
restarts and reconciles everything again. So a reconcile must be safe to run
any number of times — **idempotent** — and must only ever compare the current
world with the desired one.

### Owned vs external resources

| | Example | Cleanup |
|---|---|---|
| inside the cluster | Deployment, Service, Secret | **owner reference** → garbage collector deletes them with the owner |
| outside the cluster | a cloud database, a DNS record | **finalizer** → the API server waits for the operator to clean up |

### Spec, status, generation

`metadata.generation` increases when the spec changes (not the status).
Writing `status.observedGeneration` tells clients which spec the status
describes; `kubectl wait --for=condition=Ready` and GitOps tools rely on that.

### Server-side apply

The real-cluster code applies objects with **server-side apply** under a
field manager (`rustlearn-operator`). The API server tracks which fields each
manager owns, so the operator and, say, a HorizontalPodAutoscaler can each
own different fields of the same Deployment. It's why the operator only
compares — and sends — the fields it owns.

## Common Pitfalls

1. **Comparing whole objects** — server defaults make them always differ: an endless update loop
2. **Generating secrets on every reconcile** — passwords change every 5 minutes; read what's stored first
3. **Provisioning before adding the finalizer** — a crash in between leaks the external resource
4. **Stuck deletions** — finalizers need a running operator; uninstall operators after their resources
5. **Retrying permanent errors** — an invalid spec won't fix itself: report it in the status
6. **Changing a Deployment's selector** — it's immutable; derive it from stable labels only
7. **Writing status when nothing changed** — every write is another watch event, another reconcile

## Running This Module

```bash
cargo run  -p m22-kubernetes-operators -- demo                  # your code
cargo test -p m22-kubernetes-operators-tests --features mine    # test your code
cargo run  -p m22-kubernetes-operators-solution -- demo         # the operator on an in-memory cluster
cargo run  -p m22-kubernetes-operators-solution -- crds         # the CRDs as YAML
cargo test -p m22-kubernetes-operators-tests                    # 19 tests against the solution
```

The kube crates need **Rust 1.89 or newer** (`rustup update`). The first
build of this module takes a while: kube and the Kubernetes API types are
large.

## About This Module's Exercises

This module had no `exercises.md` in the original repository; it was written to
match the format of earlier modules and the exercises in
[the phase README](../README.md) (an operator for deployments, a CRD for app
configuration, a database operator, health checks and metrics).

- **Everything is tested without a cluster.** `FakeCluster` reproduces the
  API-server behaviour operators depend on (defaulting, generations,
  finalizers, owner-reference garbage collection) plus a simplified
  Deployment controller. It is not a full API server: no watches, no
  field-manager conflicts, no admission webhooks.
- **The real-cluster path (`controller.rs`) compiles but hasn't been run
  against a cluster in this repository's checks.** The CRDs it installs are
  generated by kube-rs and checked for the structure Kubernetes requires.
- **The "database" is simulated** (`FakeDbProvider`); a real operator would
  call a cloud API — or create a StatefulSet.

Written answers: [solution/ANSWERS.md](solution/ANSWERS.md).

## Next

[23 · Microservices](../23-microservices/)
