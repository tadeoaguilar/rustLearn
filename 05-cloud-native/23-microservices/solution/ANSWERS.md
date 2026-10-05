# Answers · 23 Microservices

## Exercise 1: Why a client-chosen `reservation_id`?

A call can time out *after* the server did the work: the reservation exists,
but the response was lost. If the client retries a request that has no
identity, the server can't tell "new request" from "the same one again" and
reserves twice. With an id chosen by the client (here, the order id), the
server stores the outcome under that id and answers retries with it. This is
the same idea as the `Idempotency-Key` header in HTTP APIs (Stripe), and it's
what makes it safe for Exercise 4 to retry `Reserve`.

## Exercise 2: Why subscribe before reading the current level?

If you read the level (5), then subscribe, a reservation that lands in
between changes the stock to 3 — but its event was published before you
subscribed. The watcher believes "5" until the *next* change. Subscribing
first means any change after the snapshot is also in the subscription; at
worst the watcher sees an event that its snapshot already included, which is
harmless because events carry the absolute level, not a delta.

## Exercise 3: No deadline on the inner call

The client gives up at 2 s and gets `DEADLINE_EXCEEDED`. Orders doesn't know:
it keeps waiting, and Inventory spends 3 s on a query nobody will read. Under
load this is how outages snowball: clients retry, each retry starts more
orphaned work, the servers spend their capacity on requests whose callers are
gone. Propagating the *remaining* time (0.5 s) lets Inventory and Orders stop
when the client stops.

## Exercise 7: What if the compensation fails?

Then the stock stays reserved for an order that will never complete. The
options, usually combined:

- **Retry the compensation** — compensations must be idempotent (Release is),
  so retrying is safe. Here, `place_order` returns the error, and when the
  client retries with the same order id the saga runs again and releases.
- **Persist the saga's state** before each step (a saga log in a database)
  so a crashed orchestrator resumes and finishes compensating on restart. The
  in-memory log in `orders.rs` shows the shape of it.
- **Time-limited reservations** — Inventory expires reservations that aren't
  confirmed within N minutes, so a lost compensation heals itself.
- **Alert and repair by hand** for whatever's left.

Compensations also aren't true rollbacks: between the reservation and its
release, other customers saw less stock. Sagas give *eventual* consistency.
