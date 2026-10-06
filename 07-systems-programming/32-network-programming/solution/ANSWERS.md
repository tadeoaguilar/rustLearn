# Answers · 32 Network Programming

## Exercise 1: 10,000 threads?

Each OS thread reserves a stack (2 MiB virtual by default for Rust's spawned
threads; 8 MiB for a main thread on Linux), costs a kernel scheduling
entity, and context switches cost microseconds. 10,000 idle clients would
mean gigabytes of address space and a scheduler juggling threads that are
almost all blocked on reads. A tokio task is a state machine of a few
hundred bytes; 10,000 waiting tasks cost a few megabytes and zero CPU until
a socket becomes readable (epoll/kqueue tells the runtime which). Threads
are simpler and perfectly fine for tens of clients; tasks scale to C10K and
beyond.

## Exercise 2: Why check after decoding?

Because the attacker controls the encoding. `/static/%2e%2e/secret` contains
no `..` until it's decoded -- and the filesystem only ever sees the decoded
path. Checking the raw string and then decoding would approve a path that
becomes `../secret`. The rule generalizes: validate the value in the form
it will be *used* in (decoded, normalized), immediately before using it.

## Exercise 4: Why subscribe first?

A broadcast receiver only sees messages sent *after* it subscribed. If the
new client subscribed after announcing itself and sending `Welcome`, any
message another client sent in between -- after the `Welcome` user list was
computed but before the subscription -- would never reach the new client:
it would join, see a user list, and silently miss that user's first words.
Subscribing first means every message after the `Welcome` snapshot is
delivered (the client's own `Joined` is filtered out).
