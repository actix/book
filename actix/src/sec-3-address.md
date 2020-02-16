# Address

Actors communicate exclusively by exchanging messages. The sending actor can optionally
wait for the response. Actors cannot be referenced directly, only by their addresses.

There are several ways to get the address of an actor. The `Actor` trait provides
two helper methods for starting an actor. Both return the address of the started actor.

Here is an example of `Actor::start()` method usage. In this example `MyActor` actor
is asynchronous and is started in the same thread as the caller - threads are covered in
the [SyncArbiter] chapter.

```rust
# extern crate actix;
# use actix::prelude::*;
struct MyActor;
impl Actor for MyActor {
    type Context = Context<Self>;
}

# fn main() {
# System::new("test");
let addr = MyActor.start();
# }
```

An async actor can get its address from the `Context` struct. The context needs to
implement the `AsyncContext` trait. `AsyncContext::address()` provides the actor's address.

```rust
# extern crate actix;
# use actix::prelude::*;
struct MyActor;
impl Actor for MyActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
       let addr = ctx.address();
    }
}
# fn main() {}
```

[SyncArbiter]: ./sec-6-sync-arbiter.md

## Message

To be able to handle a specific message the actor has to provide a
[`Handler<M>`] implementation for this message.
All messages are statically typed. The message can be handled in an asynchronous
fashion. The actor can spawn other actors or add futures or
streams to the execution context. The actor trait provides several methods that allow
 controlling the actor's lifecycle.

To send a message to an actor, the `Addr` object needs to be used. `Addr` provides several
ways to send a message.

  * `Addr::do_send(M)` - this method ignores any errors in message sending. If the mailbox
  is full the message is still queued, bypassing the limit. If the actor's mailbox is closed,
  the message is silently dropped. This method does not return the result, so if the
  mailbox is closed and a failure occurs, you won't have an indication of this.

  * `Addr::try_send(M)` - this method tries to send the message immediately. If
  the mailbox is full or closed (actor is dead), this method returns a
  [`SendError`].

  * `Addr::send(M)` - This message returns a future object that resolves to a result
  of a message handling process. If the returned `Future` object is dropped, the
  message is cancelled.

[`Handler<M>`]: https://actix.rs/actix/actix/trait.Handler.html
[`SendError`]: https://actix.rs/actix/actix/prelude/enum.SendError.html

## Recipient

Recipient is a specialized version of an address that supports only one type of message.
It can be used in case the message needs to be sent to a different type of actor.
A recipient object can be created from an address with `Addr::recipient()`.

For example recipient can be used for a subscription system. In the following example
`ProcessSignals` actor sends a `Signal` message to all subscribers. A subscriber can
be any actor that implements the `Handler<Signal>` trait.

```rust
# // This example is incomplete, so I don't think people can follow it and get value from what it's
# // trying to communicate or teach.

# #[macro_use] extern crate actix;
# use actix::prelude::*;
#[derive(Message)]
#[rtype(result = "()")]
struct Signal(usize);

/// Subscribe to process signals.
#[derive(Message)]
#[rtype(result = "()")]
struct Subscribe(pub Recipient<Signal>);

/// Actor that provides signal subscriptions
struct ProcessSignals {
    subscribers: Vec<Recipient<Signal>>,
}

impl Actor for ProcessSignals {
    type Context = Context<Self>;
}

impl ProcessSignals {

    /// Send signal to all subscribers
    fn send_signal(&mut self, sig: usize) {
        for subscr in &self.subscribers {
           subscr.do_send(Signal(sig));
        }
    }
}

/// Subscribe to signals
impl Handler<Subscribe> for ProcessSignals {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) {
        self.subscribers.push(msg.0);
    }
}
# fn main() {}
```
