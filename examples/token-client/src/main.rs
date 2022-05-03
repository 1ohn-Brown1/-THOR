use anyhow::{anyhow, Result};
use futures::{sink::SinkExt, StreamExt};
use northstar::api::client;
use std::{env, os::unix::prelude::FromRawFd};
use tokio::{
    net::{TcpStream, UnixStream},
    time::{sleep, Duration},
};
use tokio_util::codec::{Framed, LinesCodec};

/// Shared string for the token generation.
const SHARED: &str = "hello!";
/// Username - same as container name. This can also be obtained via
/// the NORTHSTAR_USERNAME env variable or Client::ident.
const USERNAME: &str = "token-client";
/// Container name of the target container.
const TARGET: &str = "token-server";

/// Some absolute random text
const TEXT: &str = "yay!";

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Connect to the runtime via NORTHSTAR_CONSOLE...
    let fd = env::var("NORTHSTAR_CONSOLE")?.parse::<i32>()?;
    let std = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) };
    std.set_nonblocking(true)?;
    let io = UnixStream::from_std(std)?;
    let mut client = client::Client::new(io, None, Duration::from_secs(5)).await?;

    // Create a token that can be used to verify `shared`. Note that there's
    // no `user` argument here. The runtime know from which container the request
    // is from.
    println!(
        "Creating token for target \"{}\" with shared \"{}\"",
        TARGET, SHARED,
    );
    let token = client.create_token(TARGET, SHARED).await?;

    // Connect to the token server
    let mut connection = TcpStream::connect("localhost:6543")
        .await
        .map(|s| Framed::new(s, LinesCodec::new()))?;

    // Encode the token for using it on the tcp connection
    let auth = format!("{} {}", USERNAME, hex::encode(token));
    // Send the authorization token to the server
    connection.send(auth).await?;

    // Send some more bytes and print the reply which should be
    // the same string as sent before.
    loop {
        println!("Sending... {}", TEXT);
        connection.send(TEXT).await?;

        let reply = connection
            .next()
            .await
            .ok_or_else(|| anyhow!("failed to receive"))??;
        assert_eq!(TEXT, reply);
        println!("Received: {}", reply);

        sleep(Duration::from_secs(1)).await;
    }
}