# Arbiter

Arbiters provide an asynchronous execution context for actors. Where an Actor
contains a Context that defines it's Actor specific execution state, Arbiters
host the environment where an actor runs.

As a result Arbiters perform a number of function. Most notably, they are able
to spawn a new OS thread, run an event loop, spawn tasks asynchronously on
that event loop, and act as helpers for asynchronous tasks.

## System and Arbiter

In all our previous code examples the function `System::new` creates an Arbiter
for your actors to run inside. When you call `start()` on your actor it is then
running inside of the System Arbiter's thread. In many cases, this is all you
will need for a program using Actix.

## Using Arbiter for resolving async events

If you aren't an expert in rust futures, Arbiter can be a helpful
and simple wrapper to resolving async events in order. Consider
we have two actors, A and B, and we want to run an event on B
only once a result from A is completed. We can use Arbiter::spawn
to assist with this task.

```rust
# extern crate actix;
# extern crate futures;
# use futures::Future;
# use actix::prelude::*;
# 
# struct SumActor {}
# 
# impl Actor for SumActor {
#     type Context = Context<Self>;
# }

struct Value(usize, usize);

# impl Message for Value {
#    type Result = usize;
# }

impl Handler<Value> for SumActor {
    type Result = usize;

    fn handle(&mut self, msg: Value, ctx: &mut Context<Self>) -> Self::Result {
        msg.0 + msg.1
    }
}

# struct DisplayActor {}
# 
# impl Actor for DisplayActor {
#     type Context = Context<Self>;
# }

struct Display(usize);

# impl Message for Display {
#    type Result = ();
# }

impl Handler<Display> for DisplayActor {
    type Result = ();

    fn handle(&mut self, msg: Display, ctx: &mut Context<Self>) -> Self::Result {
        println!("Got {:?}", msg.0);
    }
}


fn main() {
    let system = System::new("test");

    // start new actor
    let sum_addr = SumActor{}.start();
    let dis_addr = DisplayActor{}.start();

    let res = sum_addr.send(Value(6, 7));

    Arbiter::spawn(
        res.map(move |res| {

            let dis_res = dis_addr.send(Display(res));

            Arbiter::spawn(
                dis_res.map(move |_| {
                    // Shutdown gracefully now.
                    System::current().stop();
                })
                .map_err(|_| ())
            );

        }) // end res.map
        .map_err(|_| ())
    );

    system.run();
}

```

## Creating more Arbiters

Or: "Why can't I create my own Arbiters"?

Today (2019-03-15) there are lots of changes going on in the Arbiter
code, moving from actix to actix-rt, and interface changes associated.
We'll update these docs later once these changes are released.

## Assigning an Actor to a specific Arbiter

WIP

