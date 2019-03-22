# Arbiter

`Arbiter`s provide an asynchronous execution context for `Actor`s. Where an
actor contains a `Context` that defines its Actor specific execution state,
Arbiters host the environment where an actor runs.

As a result Arbiters perform a number of functions. Most notably, they are able
to spawn a new OS thread, run an event loop, spawn tasks asynchronously on that
event loop, and act as helpers for asynchronous tasks.

## System and Arbiter

In all our previous code examples the function `System::new` creates an Arbiter
for your actors to run inside. When you call `start()` on your actor it is then
running inside of the System Arbiter's thread. In many cases, this is all you
will need for a program using Actix.

While it only uses one thread, it uses the very efficient event loop pattern
which works well for asynchronous events. To handle synchronous tasks that
require heavy CPU power, it's better to avoid blocking the event loop and
instead offload the computation to other threads. For this usecase, read the
next section and consider using [`SyncArbiter`](./sec-6-sync-arbiter.md).

## The event loop

> If you're familiar with event loops in other paradigms, such as libuv (upon
> which Node runs) or tokio (upon which Actix is implemented), each Arbiter has
> a similar processing paradigm.

> Specifically, if you're familiar with how JavaScript handles asynchronicity
> and how the single-threaded model makes it so JS mostly doesn't need
> synchronization primitives, this should feel very familiar to you.

One `Arbiter` is in control of one thread with one task queue, and when tasks
get spawned in an Actor's context or using `Arbiter::spawn`, the task gets
queued up for execution in that thread.

If you pay close attention to way both `ActorFuture` and the methods in the
`Actor` trait specify their parameters, actor behavior always references both
the actor and the context as mutable: `&mut self` (or `&mut A` in `ActorFuture`)
and `&mut Self::Context`. This is because all of these tasks have **exclusive**
rights to the actor and its context. This is achieved by putting all actor logic
in the same thread, so we know no two pieces of logic involving the actor are
running concurrently without having to use synchronization primitives.

While the library is written with an asynchronous, event-driven flow, and while
the library otherwise enables concurrency, each Actor specifically runs on one
thread. Each task gets coordinated by the Arbiter to run eventually, but always
on the same thread. So when a Future is spawned on the Arbiter (using
`Arbiter::spawn` or the more common `Context::spawn`), it will execute within
the same execution context (thread), allowing the Future full access to the
state while avoiding synchronization (such as `Mutex`es).

When you create a new Arbiter, this creates a new execution context for Actors.
The new thread is available to add new Actors to it, but Actors cannot freely
move between Arbiters: they are tied to the Arbiter they were spawned in.
However, Actors on different Arbiters can still communicate with each other
using the normal `Addr`/`Recipient` methods. The method of passing messages is
agnostic to whether the Actors are running on the same or different Arbiters.

## Using Arbiter for resolving async events

If you aren't an expert in Rust Futures, Arbiter can be a helpful and simple
wrapper to resolving async events in order. Consider we have two actors, A and
B, and we want to run an event on B only once a result from A is completed. We
can use `Arbiter::spawn` to assist with this task.

```rust
# extern crate actix;
# extern crate futures;
# use actix::prelude::*;
# use futures::Future;
#
struct SumActor {}

impl Actor for SumActor {
    type Context = Context<Self>;
}

struct Value(usize, usize);

impl Message for Value {
    type Result = usize;
}

impl Handler<Value> for SumActor {
    type Result = usize;

    fn handle(&mut self, msg: Value, _ctx: &mut Context<Self>) -> Self::Result {
        msg.0 + msg.1
    }
}

struct DisplayActor {}

impl Actor for DisplayActor {
    type Context = Context<Self>;
}

struct Display(usize);

impl Message for Display {
    type Result = ();
}

impl Handler<Display> for DisplayActor {
    type Result = ();

    fn handle(&mut self, msg: Display, _ctx: &mut Context<Self>) -> Self::Result {
        println!("Got {:?}", msg.0);
    }
}

fn main() {
    let system = System::new("single-arbiter-example");

    // `Actor::start` spawns the `Actor` on the *current* `Arbiter`, which
    // in this case is the System arbiter
    let sum_addr = SumActor {}.start();
    let dis_addr = DisplayActor {}.start();

    // Define an execution flow using futures
    //
    // Start by sending a `Value(6, 7)` to our `SumActor`.
    // `Addr::send` responds with a `Request`, which implements `Future`.
    // When awaited or mapped, it will resolve to a `Result<usize, MailboxError>`.
    let execution = sum_addr
        .send(Value(6, 7))
        // `.map_err` turns `Future<usize, MailboxError>` into `Future<usize, ()>`
        //   and prints any `MailboxError`s we encounter
        .map_err(|e| {
            eprintln!("Encountered mailbox error: {:?}", e);
        })
        // Assuming the send was successful, chain another computation
        //   onto the future. Returning a future from `and_then` chains
        //   that computation to the end of the existing future.
        //
        // In this case, because the `Arbiter` is executing the futures,
        //   the result of `dis_addr.send` (also a `Response`) will be
        //   spawned and executed on the Arbiter in the same thread.
        .and_then(move |res| {
            // `res` is now the `usize` returned from

            // Once the future is complete, send the successful response (`usize`)
            // to the `DisplayActor` wrapped in a `Display
            dis_addr.send(Display(res)).map(move |_| ()).map_err(|_| ())
        })
        .map(move |_| {
            // We only want to do one computation in this example, so we
            // shut down the `System` which will stop any Arbiters within
            // it (including the System Arbiter), which will in turn stop
            // any Actor Contexts running within those Arbiters, finally
            // shutting down all Actors.
            System::current().stop();
        });

    // Spawn the future onto the current Arbiter/event loop
    Arbiter::spawn(execution);

    system.run();
}
```
