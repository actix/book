# SyncArbiter

When you normally run Actors, there are multiple Actors sharing a single
thread and event loop in the System. However for CPU bound workloads, or
highly concurrent workloads, you may wish to have a single Actor able
to run on a pool of OS threads.

This is what a SyncArbiter provides - an enviroment allowing a single Actor
to be run on multiple threads concurrently.

It's important to note a SyncArbiter can only host a single type of Actor. If
you want to have two types of Actors running, you'll need to define two SyncArbiters

## Creating a Sync Actor

When constructing your Actor, you have to change `Context` to `SyncContext`
to define that the Actor will run on a SyncArbiter instead.

```rust
# extern crate actix;
use actix::prelude::*;

struct MySyncActor;

impl Actor for MySyncActor {
    type Context = SyncContext<Self>;
}

# fn main() {
# System::new("test");
# }

```

## Starting the Sync Arbiter

Now that we have defined a Sync Actor, we can run it on a thread pool, created by
our `SyncArbiter`. We can only control the number of threads at SyncArbiter creation
time - we can't add/remove threads later.

```rust
# extern crate actix;
use actix::prelude::*;

struct MySyncActor;

impl Actor for MySyncActor {
    type Context = SyncContext<Self>;
}

# fn main() {
# System::new("test");
let addr = SyncArbiter::start(2, || MySyncActor);
# }

```

We can communicate with the addr the same way as we have with our previous Actors
that we started. We can send messages, recieve futures and results, and more.

## Sync Actor Mailboxes

Sync Actors have no Mailbox limits, but you should still use `do_send`, `try_send` and `send`
as normal to account for other possible errors or sync vs async behaviour.


