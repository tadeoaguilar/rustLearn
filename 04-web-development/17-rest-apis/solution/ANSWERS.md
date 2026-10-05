# Answers · 17 REST APIs

## Exercise 3: What goes wrong with offset pagination?

`page=N` means "skip (N-1) × per_page items". If an item is inserted (or
deleted) before the current position while a client is paging, everything
shifts: the client sees an item twice, or skips one. Offsets are also slow on
big tables — the database still walks past every skipped row.

**Keyset (cursor) pagination** fixes both: `GET /tasks?after=1234&limit=20`
means "the 20 items after id 1234 in this sort order", which a database answers
with an index seek (`WHERE id > 1234 ORDER BY id LIMIT 20`). The cost: no
"jump to page 7", and the cursor must encode every sort key. GitHub, Stripe
and Slack all use cursors for large collections.

## Exercise 4: Path versioning vs header versioning

| | Path (`/v2/tasks`) | Header (`Accept: application/vnd.tasks.v2+json`) |
|---|---|---|
| Visible / easy to try | yes — paste in a browser | no — needs a custom header |
| Caches and proxies | just work (different URL) | need `Vary: Accept` |
| URL = resource identity | the same task has two URLs | one URL per resource (purist REST) |
| Version per resource | whole API moves together | each resource can evolve separately |

Path versioning is the pragmatic default for most teams. Large API providers
often go further: Stripe and GitHub pin a *dated* version per client in a
header (`Stripe-Version`, `X-GitHub-Api-Version`). Whatever you pick, version
only on *breaking* changes — adding an optional field isn't one.
