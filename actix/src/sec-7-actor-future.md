# ActorFuture

One of the pleasant and powerful patterns of Actix is the fact that Actors can
run on only [`Arbiter`](./sec-5-arbiter) at a time. This means that Actor
methods like `Handler`s and `Actor::started` can access the Actor's state and
context mutably without using synchronization primitives.

However, not all events are based on external events like messages. The Actor
itself has the ability to spawn futures on the Context, and these futures

If you intend on spawning the Future on the Arbiter, you can give it access to
the Actor and the Context by using `ActorFuture`.

## Example

Imagine an Actor that exists to poll data from an API. Every second, it updates
its cache. Upon request, it responds with the most recent cached data.

Note: This example isn't a recommendation of when to use this pattern. It's just
an example of the mechanics.

For the sake of this example, we'll pretend we have a function that calls out to
the API, gets the response, and parses it.

```rust
# extern crate actix;
# extern crate futures;
# use actix::prelude::*;
# use futures::Future;
# use std::time::Duration;
#
# #[derive(Debug)]
# enum Weather {
#     Rain,
#     Sun,
# }
#
# fn get_weather() -> impl Future<Item = Weather, Error = ()> {
#     futures::future::ok(Weather::Rain)
# }
#
struct WeatherActor {
    current_weather: Weather,
}

impl WeatherActor {
    pub fn new() -> Self {
        Self {
            current_weather: Weather::Sun,
        }
    }

    fn update_weather(&mut self, ctx: &mut <Self as Actor>::Context) {
        // this future just returns the weather (as a plain `Future`) and is
        // unaware of any Actix concepts entirely
        let original_future = get_weather();

        // this future will resolve with both the weather *and* the actor/context
        let actor_future = original_future.into_actor(self);

        // we could also have used `wrap_future::<_, Self>`,
        // but `.into_actor` is more concise
        ctx.spawn(actor_future.map(
            // now, in `.map`'s callback, we can access the actor
            // and context directly, obviating the need to put
            // `WeatherActor::current_weather` in a `Mutex`
            |weather: Weather, act: &mut Self, _ctx: &mut <Self as Actor>::Context| {
                act.current_weather = weather;
            },
        ));
    }
}

impl Actor for WeatherActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Every second, update actor's cache
        ctx.run_interval(
            Duration::from_secs(1),
            |act: &mut Self, ctx: &mut Self::Context| {
                act.update_weather(ctx);
            },
        );

        ctx.run_interval(
            Duration::from_secs(1),
            |act: &mut Self, _ctx: &mut Self::Context| {
                // Again, we don't need to do anything special to get or
                // set the weather directly on the Actor instance
                println!("The weather is: {:?}", act.current_weather);
            },
        );
    }
}
#
# fn main() {
#     let system = System::new("actor-future-example");
#
#     let _weather_addr = WeatherActor::new().start();
#
#     system.run();
# }
```
