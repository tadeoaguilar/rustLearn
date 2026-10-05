# Answers · 22 Kubernetes Operators

## Exercise 1: Why must the status be a subresource?

With the `status` subresource enabled, the main endpoint ignores changes to
`.status` and the `/status` endpoint ignores everything else. So:

- users (`kubectl apply`) can't overwrite the status, and the operator's status
  writes can't clobber a user's concurrent spec change;
- status writes don't bump `metadata.generation`, which then means "the spec
  changed" — the basis for `observedGeneration`;
- RBAC can grant the operator `apps/status` separately from `apps`.

## Exercise 2: Why doesn't a ConfigMap change restart the pods?

A Deployment rolls out new pods only when its **pod template** changes.
Mounted ConfigMaps are eventually updated in place inside running pods (after
up to a minute or so), but most programs read configuration once at startup,
and environment variables from a ConfigMap never update. Putting a hash of the
config into a pod-template annotation makes every config change a template
change, so the Deployment performs a normal rolling update — with readiness
checks, and a rollback if the new config breaks the app.

## Exercise 3: What happens with `desired != observed`?

The observed object always contains server-filled fields (defaults, a
cluster IP, uids, status), so it never equals the desired one. Every reconcile
writes; every write creates a watch event; every event triggers a reconcile:
a hot loop that burns API-server capacity and floods audit logs. With
server-side apply against a real cluster, a write of unchanged owned fields is
a no-op, which hides much of the cost — but the loop still exists as reads,
requests and reconciles. Comparing only owned fields stops it at the source.

## Exercise 5

**Why add the finalizer before provisioning?** Reconciles can crash at any
line. If the database were created first, a crash before the finalizer was
added leaves an external database with no finalizer — and a `kubectl delete`
in that window removes the Database object immediately, so nothing will ever
clean the database up. Adding the finalizer first means the worst case is a
finalizer with nothing to clean up, which the delete path handles
(`provider.delete` is idempotent).

**What if the operator is uninstalled while Databases exist?** Nothing
removes the finalizers, so `kubectl delete database ...` hangs forever and
namespace deletion gets stuck in `Terminating`. The fixes: delete the custom
resources *before* the operator, or (knowing the external resource is leaked)
remove the finalizer by hand:
`kubectl patch rdb orders-db --type=merge -p '{"metadata":{"finalizers":null}}'`.

## Bonus

- **`kubectl scale ... --replicas=5`** — the scale changes the Deployment,
  which the controller watches (`.owns(...)`), so the App is reconciled within
  moments and replicas go back to the spec. If you *want* someone else (an
  HPA) to own replicas, stop setting them in the operator — server-side apply
  then lets the other manager own the field.
- **Deleting a Database while the operator is stopped** — the API server sets
  `deletionTimestamp` and waits; the object sits in the list until the
  operator starts again, sees it, cleans up and removes the finalizer. That's
  the whole point: the cleanup can't be skipped.
