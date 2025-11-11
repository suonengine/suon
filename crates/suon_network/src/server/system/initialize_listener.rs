use bevy::{prelude::*, tasks::IoTaskPool};
use smol::block_on;

use crate::server::{
    connection::{incoming::IncomingConnections, throttle::Throttle},
    settings::Settings,
};

/// Initializes the listener for incoming client connections.
pub(crate) fn initialize_listener(
    settings: Res<Settings>,
    incoming_connections: Res<IncomingConnections>,
    throttle: Res<Throttle>,
) {
    let address = settings.address;
    let use_nagle_algorithm = settings.use_nagle_algorithm;
    let throttle = throttle.clone();
    let incoming_connections = incoming_connections.clone();

    // Attempt to bind a listener to the configured address.
    let listener = block_on(smol::net::TcpListener::bind(address))
        .unwrap_or_else(|err| panic!("Failed to bind server listener on {address}. {err}"));

    IoTaskPool::get()
        .spawn(async move {
            info!("Listening for incoming client connections on {}", address);

            // Loop to continuously accept incoming connections
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        trace!("Accepted connection from {}", addr);

                        // Apply throttle policy to prevent abuse or excessive connections.
                        if let Err(err) = throttle.attempt_connection(&addr) {
                            warn!(
                                "Connection from {} rejected by throttle policy: {:?}",
                                addr, err
                            );
                            continue;
                        }

                        info!("New connection accepted from {}", addr);

                        // Configure the stream for low-latency communication
                        // by setting Nagle's algorithm according to the settings.
                        if let Err(err) = stream.set_nodelay(use_nagle_algorithm) {
                            warn!("Failed to set nodelay for {}: {:?}", addr, err);
                        } else {
                            debug!(
                                "Nagle's algorithm set to {} for {}",
                                use_nagle_algorithm, addr
                            );
                        }

                        // Attempt to enqueue the new connection for further processing.
                        if let Err(err) = incoming_connections.send(stream) {
                            error!(
                                "Failed to enqueue incoming connection from {}: {:?}",
                                addr, err
                            );
                            break;
                        } else {
                            trace!("Connection from {} enqueued successfully", addr);
                        }
                    }
                    Err(err) => {
                        error!("Failed to accept client connection: {:?}", err);
                    }
                }
            }
        })
        .detach();
}
