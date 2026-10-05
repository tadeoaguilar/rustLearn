# Exercises: Kubernetes Operators

An **operator** encodes what a human operator knows about running an
application into a program that runs inside the cluster. Users write a short
custom resource; the operator turns it into everything needed and keeps it
that way. You'll build one with [kube-rs](https://kube.rs/) for two custom
resources:

```yaml
kind: App        # -> ConfigMap + Deployment + Service, rolled on config change
kind: Database   # -> an external database (with a finalizer) + a credentials Secret
```

**Setup**: every test runs against `FakeCluster` (`cluster.rs`, provided), an
in-memory API server that behaves like the real one where it matters:
defaults, generations, finalizers, garbage collection, and a simulated
Deployment controller. A real cluster (kind, Docker Desktop, minikube) is
optional — see GETTING_STARTED.md.

---

## Exercise 1: Custom Resource Definitions

**Difficulty**: Medium
**Time**: 40 minutes

**Learning Objectives**:
- Derive a CRD from Rust types with `#[derive(CustomResource)]`
- Put validation where the API server enforces it

`crd.rs` has the `App` and `Database` types. Complete them:

1. Schema validation (`#[schemars(...)]`): `replicas` 0–50, `port` 1–65535,
   `image` non-empty; `storageGb` 1–1000, `version` non-empty
2. Printer columns for `kubectl get apps` (Image, Desired, Ready, Phase) and
   `kubectl get databases` (Engine, Phase, Endpoint)
3. `crds_yaml()` — both CRDs as one multi-document YAML string
4. `validate_app(&AppSpec) -> Result<(), Vec<String>>` for what a schema
   can't say: the image has a tag or digest and isn't `:latest`; env names
   are identifiers; config keys are valid file names. Report every problem.

**Question**: why must the status be a *subresource*?

---

## Exercise 2: The Desired State

**Difficulty**: Medium
**Time**: 50 minutes

`resources.rs` — pure functions from an `App` to its children:

```rust
pub fn desired_configmap(app: &App) -> ConfigMap;    // "<name>-config", data = spec.config
pub fn desired_deployment(app: &App) -> Deployment;  // image, env, port, probes, config volume
pub fn desired_service(app: &App) -> Service;        // port 80 -> the "http" container port
pub fn config_hash(config: &BTreeMap<String, String>) -> String;  // 16 hex digits
```

Every child gets the `app.kubernetes.io/*` labels and a controller **owner
reference** (`app.controller_owner_ref(&())`). The pod template carries the
config hash as an annotation, runs as non-root, and probes `/healthz` and
`/readyz` — module 21's service.

**Question**: why does changing the ConfigMap not restart the pods, and how
does the annotation fix that?

---

## Exercise 3: Planning Without Hot Loops

**Difficulty**: Hard
**Time**: 50 minutes

```rust
pub fn plan(desired: &Desired, observed: &Observed) -> Plan;
// Plan { configmap: Option<Change>, deployment: Option<Change>, service: Option<Change> }
// Change::Create | Change::Update(vec!["replicas", "image", ...])
```

The API server adds defaults to everything you create (`protocol: TCP`, probe
timeouts, `revisionHistoryLimit`, a cluster IP...). Compare only the fields
the operator owns: replicas, image, env, ports, probe paths, the config hash
(Deployment); data (ConfigMap); selector and ports (Service).

The test round-trips desired objects through `FakeCluster` and expects an
empty plan. **Question**: what happens with `desired != observed` instead?

---

## Exercise 4: Status and Conditions

**Difficulty**: Medium
**Time**: 30 minutes

`status::app_status(app, deployment) -> AppStatus`:

| Situation | Phase | Ready condition |
|---|---|---|
| no Deployment yet | Pending | False, `NotCreated` |
| `ProgressDeadlineExceeded` | Degraded | False |
| controller has seen the latest generation, all replicas ready and updated | Ready | True, `"3/3 replicas ready"` |
| otherwise | Progressing | False, `"1/3 replicas ready"` |

Always set `observed_generation` from the App's `metadata.generation`.

---

## Exercise 5: A Database Operator With a Finalizer

**Difficulty**: Hard
**Time**: 60 minutes

A `Database` lives outside the cluster (`DbProvider`, a fake is provided),
so owner references can't clean it up. Implement in `database.rs` the
helpers (`with_finalizer`, `without_finalizer`, `credentials_secret`,
`password_from`, `generate_password`) and in `reconcile.rs`:

```
reconcile_database:
  being deleted?  -> provider.delete -> remove our finalizer -> done
  no finalizer?   -> add it FIRST
  provider.ensure -> Invalid(..) -> status Degraded (don't retry)
                  -> Unavailable -> Err (retry)
  Secret "<name>-credentials": reuse the stored password, else generate one
  status Ready with endpoint and secret name
```

**Questions**: why add the finalizer *before* provisioning? What happens if
the operator is uninstalled while Databases with finalizers exist?

---

## Exercise 6: Reconciling Apps

**Difficulty**: Hard
**Time**: 45 minutes

```rust
pub async fn reconcile_app<C: Cluster>(app: &App, ctx: &Context<C>) -> Result<Outcome, Error>;
```

Validate (invalid → Degraded status, create nothing, `AwaitChange`); read
the children; `plan`; apply only what the plan says (ConfigMap first); write
the status only if it changed; requeue in 5 s while rolling out, 5 min once
ready.

`run_until_converged` (in `reconcile.rs`) runs every reconciler and the
fake Deployment controller in rounds until nothing changes. The tests
check: convergence; **zero writes** once converged; config changes rolling
the pods; a hand-scaled Deployment being put back; deletion.

---

## Exercise 7: Retries and Metrics

**Difficulty**: Medium
**Time**: 35 minutes

`metrics.rs`:
- `Backoff` — per-object exponential backoff: 1 s, 2 s, 4 s... capped;
  reset on success
- `Metrics` — count reconciles by kind and result, sum durations;
  `render()` in the Prometheus text format:

```
# TYPE rustlearn_operator_reconciles_total counter
rustlearn_operator_reconciles_total{kind="App",result="ok"} 12
```

The test injects API and provider failures and expects convergence anyway.

---

## Bonus Challenge: A Real Cluster

**Difficulty**: Medium
**Time**: 60 minutes

`controller.rs` (provided) implements `Cluster` with kube-rs and runs your
reconcilers in `kube::runtime::Controller`s. Create a cluster, install the
CRDs, run the operator from your machine, and apply `deploy/examples/`
(GETTING_STARTED.md has the commands). Then:

- `kubectl scale deployment web --replicas=5` — how fast does it come back?
- `kubectl delete rdb orders-db` — watch the finalizer with `kubectl get rdb -w`
- stop the operator, delete a Database, run `kubectl get rdb` — why is it stuck?

---

## Check Your Understanding

- [ ] Explain the reconcile loop: level-based, idempotent, requeued
- [ ] Generate a CRD with a validated schema from Rust types
- [ ] Use owner references and know what they clean up
- [ ] Use a finalizer for external resources — and its failure modes
- [ ] Explain why naive diffs cause hot loops
- [ ] Report status with conditions and `observedGeneration`
- [ ] Back off per object on failures

---

## Additional Resources

- [kube-rs](https://kube.rs/) and [controller-rs](https://github.com/kube-rs/controller-rs), the reference example
- [Kubernetes: Operator pattern](https://kubernetes.io/docs/concepts/extend-kubernetes/operator/)
- [Custom resources](https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definitions/)
- [Finalizers](https://kubernetes.io/docs/concepts/overview/working-with-objects/finalizers/) and [garbage collection](https://kubernetes.io/docs/concepts/architecture/garbage-collection/)
- [API conventions: spec, status, conditions](https://github.com/kubernetes/community/blob/master/contributors/devel/sig-architecture/api-conventions.md)
