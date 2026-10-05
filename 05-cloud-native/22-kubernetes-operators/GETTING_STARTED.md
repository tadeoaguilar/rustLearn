# Getting Started with 22 · Kubernetes Operators

## Quick Start

All commands run from the **repository root**. The kube crates need Rust
1.89+ (`rustc --version`; `rustup update` if older).

```bash
cargo run -p m22-kubernetes-operators-solution -- demo
```

creates an App on the in-memory cluster and walks it through a config change,
a manual scale (repaired), a bad image, an invalid spec, then a Database with
its finalizer, and prints the operator's metrics.

## What Is Already Here

```
22-kubernetes-operators/
├── README.md, exercises.md, GETTING_STARTED.md
├── deploy/
│   ├── crds.yaml            # generated: cargo run -p m22-kubernetes-operators-solution -- crds
│   ├── rbac.yaml            # ServiceAccount + ClusterRole for running in-cluster
│   └── examples/            # app.yaml, database.yaml
├── exercise/src/            # ← YOUR WORKSPACE (package m22-kubernetes-operators)
│   ├── crd.rs               #   Ex 1
│   ├── resources.rs         #   Ex 2
│   ├── plan.rs              #   Ex 3
│   ├── status.rs            #   Ex 4
│   ├── database.rs          #   Ex 5 (FakeDbProvider provided)
│   ├── reconcile.rs         #   Ex 5, 6 (run_until_converged provided)
│   ├── metrics.rs           #   Ex 7
│   ├── cluster.rs           #   provided: Cluster trait + FakeCluster
│   └── controller.rs        #   provided: real-cluster wiring
├── solution/                # ← REFERENCE (m22-kubernetes-operators-solution) + ANSWERS.md
└── tests/                   # ← 19 tests (m22-kubernetes-operators-tests)
```

## The Commands You Need

```bash
cargo run  -p m22-kubernetes-operators -- demo
cargo test -p m22-kubernetes-operators-tests --features mine
cargo test -p m22-kubernetes-operators-tests --features mine ex3_
cargo run  -p m22-kubernetes-operators -- crds          # your CRDs
cargo test -p m22-kubernetes-operators-tests            # the solution: always green
```

## On a Real Cluster (Optional)

You need a cluster and `kubectl` pointing at it. Any of:

```bash
kind create cluster                     # https://kind.sigs.k8s.io (needs Docker)
# or: Docker Desktop -> Settings -> Kubernetes -> Enable
# or: minikube start / k3d cluster create
kubectl get nodes                       # check
```

Then:

```bash
# 1. Install the CRDs (yours or the solution's)
cargo run -q -p m22-kubernetes-operators-solution -- crds | kubectl apply -f -
kubectl get crd | grep rustlearn

# 2. Run the operator on your machine, using your kubeconfig
cargo run -p m22-kubernetes-operators-solution -- run

# 3. In another terminal, create resources and watch
kubectl apply -f 05-cloud-native/22-kubernetes-operators/deploy/examples/
kubectl get rapp,rdb -w
kubectl get deploy,svc,cm,secret -l app.kubernetes.io/managed-by=rustlearn-operator
kubectl wait --for=condition=Ready app/web --timeout=120s

# 4. Poke it
kubectl scale deployment web --replicas=5          # put back to 2
kubectl edit rapp web                               # change config: pods roll
kubectl delete rdb orders-db                        # finalizer: cleanup, then gone

# 5. Clean up
kubectl delete -f 05-cloud-native/22-kubernetes-operators/deploy/examples/
kubectl delete -f 05-cloud-native/22-kubernetes-operators/deploy/crds.yaml
```

The example App uses `rustlearn/m21:scratch` from module 21; with kind, load it
with `kind load docker-image rustlearn/m21:scratch`. Any image serving
`/healthz` and `/readyz` on the port works.

Running the operator *inside* the cluster needs an image of it and
`deploy/rbac.yaml`; building that image works like module 21's Dockerfiles.

## If You Get Stuck

1. **`owner_references` is None** — `controller_owner_ref` needs `metadata.uid`; objects only get one once stored. Tests give theirs a uid.
2. **The plan is never empty** — you're comparing something the server defaults (ports' `protocol`, probe timeouts). Compare normalised owned fields only.
3. **`run_until_converged` returns `Err`** — something writes on every round: a status that changes each time, a password regenerated, a diff that never settles.
4. **The Database is never deleted** — after `provider.delete`, remove *your* finalizer with `set_finalizers`; the fake deletes the object when the list is empty.
5. **`cargo test` says the CRDs are stale** — only for the solution: regenerate `deploy/crds.yaml` with `-- crds`.
6. Compare with `solution/src/` — same file and function names.
