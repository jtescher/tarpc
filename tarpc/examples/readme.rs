// Copyright 2018 Google LLC
//
// Use of this source code is governed by an MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT.

use futures::{
    future::{self, Ready},
    prelude::*,
};
use std::io;
use tarpc::{
    client, context,
    server::{BaseChannel, Channel},
};

/// This is the service definition. It looks a lot like a trait definition.
/// It defines one RPC, hello, which takes one arg, name, and returns a String.
#[tarpc::service]
pub trait World {
    async fn hello(name: String) -> String;
}

/// This is the type that implements the generated World trait. It is the business logic
/// and is used to start the server.
#[derive(Clone)]
struct HelloServer;

impl World for HelloServer {
    // Each defined rpc generates two items in the trait, a fn that serves the RPC, and
    // an associated type representing the future output by the fn.

    type HelloFut = Ready<String>;

    fn hello(self, _: context::Context, name: String) -> Self::HelloFut {
        future::ready(format!("Hello, {}!", name))
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // tarpc_json_transport is provided by the associated crate json_transport. It makes it
    // easy to start up a serde-powered JSON serialization strategy over TCP.
    let mut transport = tarpc::json_transport::listen("0.0.0.0:0").await?;
    let addr = transport.local_addr();

    let server = async move {
        // For this example, we're just going to wait for one connection.
        let client = transport.next().await.unwrap().unwrap();

        // `Channel` is a trait representing a server-side connection. It is a trait to allow
        // for some channels to be instrumented: for example, to track the number of open connections.
        // BaseChannel is the most basic channel, simply wrapping a transport with no added
        // functionality.
        BaseChannel::with_defaults(client)
            // serve_world is generated by the tarpc::service attribute. It takes as input any type
            // implementing the generated World trait.
            .respond_with(HelloServer.serve())
            .execute()
            .await;
    };
    tokio::spawn(server);

    let transport = tarpc::json_transport::connect(addr).await?;

    // WorldClient is generated by the tarpc::service attribute. It has a constructor `new` that
    // takes a config and any Transport as input.
    let mut client = WorldClient::new(client::Config::default(), transport).spawn()?;

    // The client has an RPC method for each RPC defined in the annotated trait. It takes the same
    // args as defined, with the addition of a Context, which is always the first arg. The Context
    // specifies a deadline and trace information which can be helpful in debugging requests.
    let hello = client.hello(context::current(), "Stim".to_string()).await?;

    eprintln!("{}", hello);

    Ok(())
}
