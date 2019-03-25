# Streams

Actix allows the handling of Tokio Streams to act as a source of events
or messages to actors. This is commonly used with networking servers
and clients, for communication over TCP, UDP and UnixSockets.

At it's core, Actix when using streams provides a mechanism for taking
whole message frames from a stream, and converting these to messages which
are sent to your Actor's handlers.

To show how these parts are created, we'll create a TCP echo server.

## Wrapping a stream of events

A stream provides a series of frames to Actix which reperesent some event
or input provided by the stream. The first method to handle these inputs
from a stream, is to asynchronously map over the inputs and turn these into
messages that can be sent to an Actor.

The best example for this is a TcpListener, which provides a stream
of new connections coming in from clients. As each client connects, we
can wrap the new connection into a message, which is sent to a handler
to process.

```rust
extern crate futures;
extern crate tokio;
extern crate tokio_io;
extern crate tokio_tcp;

extern crate actix;

use std::net;
use std::str::FromStr;
use std::io;
use bytes::{BufMut, BytesMut};

use actix::prelude::*;
use futures::Stream;
use tokio_codec::{Decoder, Encoder, FramedRead};
use tokio_io::AsyncRead;
use tokio_io::io::WriteHalf;
use tokio_tcp::{TcpListener, TcpStream};

struct AcceptServer;

impl Actor for AcceptServer {
    type Context = Context<Self>;
}

#[derive(Message)]
struct TcpConnect(pub TcpStream, pub net::SocketAddr);

impl Handler<TcpConnect> for AcceptServer {
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        println!("Incoming connection from {:?}", msg.1);
        // As we do nothing with the incomming connection here,
        // it is dropped and closed.
    }
}

fn main() {
    actix::System::run(|| {
        // Create server listener
        let addr = net::SocketAddr::from_str("127.0.0.1:4321").unwrap();
        let listener = TcpListener::bind(&addr).unwrap();

        AcceptServer::create(|ctx| {
            // Add message stream allows taking events from a stream, mapping them
            // into Messages, and then sending them to Actors for handling.
            ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|st| {
                let addr = st.peer_addr().unwrap();
                // Everytime a new connection is made, the st (new accepted connection) is
                // sent as a TcpConnect message to the AcceptServer Actor.
                TcpConnect(st, addr)
            }));
            AcceptServer {}
        });

        println!("Running tcp echo server on 127.0.0.1:4321");
    });
}
```

## Using a stream as a source of messages

Now that we are able to accept new connections, we need to do something with them.
We can use our accepted connection as a stream of frames, where those frames are
sent as messages to an actor. Each connection is associated to a unique actor.

# Writing a Codec

First, we need to write a codec capable of converting bytes into whole frames
that will eventually become messages. Codecs have two haves - the Decoder, that takes
user input and returns frames, and an encoder that takes frames and converts them
to bytes to be transmitted to the client.

An important aspect of our codec and it's ability to be used with Actix is that
the frames must implement the trait Message. If the frame doesn't implement Message you may need
to use `add_message_stream` instead to perform the frame to message encapsulation.


```rust
extern crate bytes;
use bytes::{BufMut, BytesMut};

struct Bytes(Vec<u8>);

impl Message for Bytes {
    type Result = ();
}

struct ByteCodec;

impl Decoder for ByteCodec {
    type Item = Bytes;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>, io::Error> {
        if buf.len() > 0 {
            let len = buf.len();
            let b = Bytes(buf.split_to(len).to_vec());
            // We have a full frame, return it.
            Ok(Some(b))
        } else {
            // We have no data, so return that we don't have enough to complete
            // a full frame. It's important you handle this case.
            Ok(None)
        }
    }
}

impl Encoder for ByteCodec {
    type Item = Bytes;
    type Error = io::Error;

    fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), io::Error> {
        buf.reserve(data.0.len());
        buf.put(data.0);
        // Indicate that we correctly encoded the frame and it can now be transmitted.
        Ok(())
    }
}
```

For more about the implementation of Codecs, see ...

# Using the Codec with messages

Now that we have a way to transform our bytes into Messages that our actor can understand,
we can add the read half of the connection as a stream of Messages into the context of our actor.
We'll extend our example with our Codec and the following EchoServer implementation.

```rust
# extern crate bytes;
# extern crate futures;
# extern crate tokio;
# extern crate tokio_io;
# extern crate tokio_tcp;

# extern crate actix;

# use std::net;
# use std::str::FromStr;
# use std::io;
# use bytes::{BufMut, BytesMut};

# use actix::prelude::*;
# use futures::Stream;
use tokio_codec::{Decoder, Encoder, FramedRead};
use tokio_io::AsyncRead;
use tokio_io::io::WriteHalf;
# use tokio_tcp::{TcpListener, TcpStream};

# struct Bytes(Vec<u8>);
# 
# impl Message for Bytes {
#     type Result = ();
# }
# 
# struct ByteCodec;
# 
# impl Decoder for ByteCodec {
#     type Item = Bytes;
#     type Error = io::Error;
# 
#     fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Bytes>, io::Error> {
#         if buf.len() > 0 {
#             let len = buf.len();
#             let b = Bytes(buf.split_to(len).to_vec());
#             Ok(Some(b))
#         } else {
#             Ok(None)
#         }
#     }
# }
# 
# impl Encoder for ByteCodec {
#     type Item = Bytes;
#     type Error = io::Error;
# 
#     fn encode(&mut self, data: Bytes, buf: &mut BytesMut) -> Result<(), io::Error> {
#         buf.reserve(data.0.len());
#         buf.put(data.0);
#         Ok(())
#     }
# }

struct EchoServer {
    framed: actix::io::FramedWrite<WriteHalf<TcpStream>, ByteCodec>,
}

impl Actor for EchoServer {
    type Context = Context<Self>;
}

impl EchoServer {
    pub fn new(framed: actix::io::FramedWrite<WriteHalf<TcpStream>, ByteCodec>) -> Self {
        EchoServer {
            framed: framed,
        }
    }
}

impl actix::io::WriteHandler<io::Error> for EchoServer {}

impl StreamHandler<Bytes, io::Error> for EchoServer {
    fn handle(&mut self, msg: Bytes, _: &mut Context<Self>) {
        // Return the msg to the client as we received it. This is the
        // "echo" part of an echo server!
        // In a more complete application you would like match on the msg
        // and make decisions about what next action to perform.
        self.framed.write(msg);
    }
}

# struct AcceptServer;
# 
# impl Actor for AcceptServer {
#     type Context = Context<Self>;
# }
# 
# #[derive(Message)]
# struct TcpConnect(pub TcpStream, pub net::SocketAddr);

impl Handler<TcpConnect> for AcceptServer {
    type Result = ();

    fn handle(&mut self, msg: TcpConnect, _: &mut Context<Self>) {
        println!("Incoming connection from {:?}", msg.1);
        // For each incoming connection we create `EchoServer` actor
        // with out chat server address. This means there will be
        // many EchoServer actors, one for each client!
        EchoServer::create(move |ctx| {
            // Split the "stream" into a read stream and a write sink.
            let (r, w) = msg.0.split();
            // This adds the read stream to `ctx`: Every decoded frame will be
            // sent to the EchoServer as a message for handling.
            EchoServer::add_stream(FramedRead::new(r, ByteCodec), ctx);
            // This now creates the actor with the write sink and the
            // reading stream `ctx`. The actor can write to the write sink
            // to respond to the client. When the client disconnects
            // the EchoServer actor is stopped for that client instance.
            EchoServer::new(actix::io::FramedWrite::new(w, ByteCodec, ctx), msg.1)
        });
    }
}

# fn main() {
#     actix::System::run(|| {
#         // Create server listener
#         let addr = net::SocketAddr::from_str("127.0.0.1:4321").unwrap();
#         let listener = TcpListener::bind(&addr).unwrap();
# 
#         AcceptServer::create(|ctx| {
#             // Add message stream allows taking events from a stream, mapping them
#             // into Messages, and then sending them to Actors for handling.
#             ctx.add_message_stream(listener.incoming().map_err(|_| ()).map(|st| {
#                 let addr = st.peer_addr().unwrap();
#                 // Everytime a new connection is made, the st (new accepted connection) is
#                 // sent as a TcpConnect message to the AcceptServer Actor.
#                 TcpConnect(st, addr)
#             }));
#             AcceptServer {}
#         });
# 
#         println!("Running tcp echo server on 127.0.0.1:4321");
#     });
# }
```

It's a common pattern that for each connection, a unique Actor is spawned to represent
that connection, and to interact with and message that connection. 

For a further example see the examples/chat server in the Actix source code. This goes
further and shows how you would implement a simple chat service with streams.

# Using TLS as a stream


