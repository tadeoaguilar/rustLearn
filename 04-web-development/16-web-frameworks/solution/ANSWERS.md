# Answers · 16 Web Frameworks

## Exercise 1: Why does every method take `&self`?

Because the store is *shared*: every request handler, on every worker thread,
holds a clone of the same `NoteStore`. Shared access means `&self`; mutation
goes through the interior `RwLock` (and the `AtomicU64` for ids). A
`&mut self` API would need exclusive access, which a server handling many
requests at once can never give.

## Exercise 3

**`HttpServer::new` takes a closure, not an `App`. Why?**

Actix-web runs one single-threaded runtime per worker thread, and each worker
gets its *own* `App` instance. The closure is a factory called once per worker
— which is why everything captured by it must be cloneable (`web::Data` is an
`Arc` inside) and why per-worker state is possible.

**Can an Actix server run inside `#[tokio::main]`?**

Yes, since actix-web 4: `server.await` or `tokio::spawn(server)` works from a
Tokio runtime, and the workers still create their own runtimes. The solution's
`demo` and `bench` run both frameworks in one Tokio program this way.

## Exercise 5: Who escaped the `<b>` tags?

askama. In a template whose file name ends in `.html`, every `{{ expression }}`
is HTML-escaped by default (`<` becomes `&#60;`). To output trusted HTML you
must opt out explicitly with the `|safe` filter — which is exactly the
decision you want to be visible in code review.

## Exercise 7

**Which is faster?** On the machine this solution was written on (release
build, 20,000 requests, 50 concurrent): Actix-web ~103k req/s, Axum ~93k
req/s, p99 under 1 ms for both. Your numbers will differ, and the order can
flip between runs.

**Why can't this benchmark tell you which is faster in production?** The
client competes with the server for the same CPU cores; the handler does
almost nothing, so you're measuring framework overhead that a real database
query (milliseconds) would dwarf; and there's no network, TLS, logging or
realistic payload. Benchmarks like this only show that both frameworks are far
from being your bottleneck.
