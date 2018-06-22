Actix is an actor model for Rust very similar to how Akka is also an actor model for Scala & Java. This page aims to introduce Actix from the perspective of someone familiar with Akka.

# Typed Actors

The most obvious difference is that Actix actors are typed. If you try to send an actor a message that it can't handle, the program won't compile.

## The `receive` Method vs The `Handler` Trait

In Akka, every actor must implement the `receive` method which accepts `Any` (all messages). A runtime exception is thrown if the actor can't handle a message.

In Actix, you have to implement `Handler<M: Message>` for every possible message that the actor handles. 

```
#[macro_use] extern crate actix;
use actix::prelude::*;

#[derive(Message)]
struct Ping { pub id: usize }

struct MyActor;

// Implement Handler<M: Message> for the Ping message
impl Handler<Ping> for MyActor {
    type Result = ();

    fn handle(&mut self, msg: Ping, ctx: &mut Context<Self>) {
        println!("Ping received {:?}", msg.id);
    }
}

fn main() {
    let system = System::new("test");
    let my_addr: Addr<MyActor> = MyActor{}.start();
    system.run();
    my_addr.do_send(Ping{id: 1});
    system.stop();
}
```

The main drawback to Actix' approach is that message handling is spread over many methods & impls which makes it less simple to get a listing of all messages it responds to. On the other hand, the compiler won't allow you to send an actor a message that it can't handle. 

In terms of extensibility, you can make an Actix actor respond to new messages without touching the original code (an artifact of Rust traits acting like type classes). In Akka, you must always modify the internals of the actor whenever it needs to support a new message.


## `ActorRef` vs`Addr`

Unlike Akka, Actix's `Addr<T: Actor>` is a generic type that provides compile-time type safety. The 2nd generic parameter provides this compile time safety. Similar to Akka, you can pass Actix Addr objects between actors and threads. Unlike Akka, the `do_send` method can fail (Akka uses unbounded queues by default, which have a strange failure mode - they fill up all available memory rather than reporting that the actor is unsendable).

## Supervisor

This is also called `Supervisor<T: Actor>` in Actix, however, in Actix, actors aren't supervised by default. Actors have to implement the `Supervised` trait which has a method called `restarting`. In Actix, the suervisor calls `restarting` rather than allocating a new actor.

