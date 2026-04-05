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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::{Ipv4Addr, SocketAddr},
        thread,
        time::{Duration, Instant},
    };

    fn available_address() -> SocketAddr {
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
            .expect("the port-probing listener should bind successfully");
        let addr = listener
            .local_addr()
            .expect("the port-probing listener should expose a local address");
        drop(listener);
        addr
    }

    #[test]
    fn should_accept_clients_and_enqueue_their_streams() {
        let address = available_address();
        let settings = Settings {
            address,
            use_nagle_algorithm: false,
            ..Settings::default()
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(settings);
        app.init_resource::<IncomingConnections>();
        app.insert_resource(Throttle::new(settings));
        app.add_systems(Update, initialize_listener);

        app.update();

        let client = wait_for_connection(address);
        let queued = wait_for_stream(app.world().resource::<IncomingConnections>());
        let peer_addr = queued
            .peer_addr()
            .expect("the accepted stream should expose its peer address");

        assert_eq!(
            peer_addr,
            client
                .local_addr()
                .expect("the client stream should expose its local address"),
            "initialize_listener should enqueue the socket accepted from the listener task"
        );
    }

    #[test]
    fn should_enqueue_multiple_allowed_connections() {
        let address = available_address();
        let settings = Settings {
            address,
            ..Settings::default()
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(settings);
        app.init_resource::<IncomingConnections>();
        app.insert_resource(Throttle::new(settings));
        app.add_systems(Update, initialize_listener);

        app.update();

        let first = wait_for_connection(address);
        let second = wait_for_connection(address);

        let queued = wait_for_all_streams(app.world().resource::<IncomingConnections>(), 2);

        assert_eq!(
            queued.len(),
            2,
            "initialize_listener should enqueue every accepted connection that passes the \
             listener loop"
        );
        assert_eq!(
            queued[0]
                .peer_addr()
                .expect("The queued stream should expose its peer address"),
            first
                .local_addr()
                .expect("The first client stream should expose its local address"),
            "The queued connection should correspond to the allowed client"
        );
        assert_eq!(
            queued[1]
                .peer_addr()
                .expect("The queued stream should expose its peer address"),
            second
                .local_addr()
                .expect("The second client stream should expose its local address"),
            "The second queued connection should correspond to the second accepted client"
        );

        drop(second);
    }

    fn wait_for_connection(address: SocketAddr) -> smol::net::TcpStream {
        let deadline = Instant::now() + Duration::from_secs(2);

        loop {
            match smol::block_on(smol::net::TcpStream::connect(address)) {
                Ok(stream) => return stream,
                Err(err) if Instant::now() < deadline => {
                    debug!("listener not ready yet during test: {err}");
                    thread::sleep(Duration::from_millis(10));
                }
                Err(err) => panic!("the listener did not accept connections in time: {err}"),
            }
        }
    }

    fn wait_for_stream(incoming_connections: &IncomingConnections) -> smol::net::TcpStream {
        let deadline = Instant::now() + Duration::from_secs(2);

        loop {
            if let Some(stream) = incoming_connections.read().into_iter().next() {
                return stream;
            }

            assert!(
                Instant::now() < deadline,
                "initialize_listener did not enqueue an accepted stream in time"
            );

            thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_all_streams(
        incoming_connections: &IncomingConnections,
        expected: usize,
    ) -> Vec<smol::net::TcpStream> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut streams = Vec::new();

        loop {
            streams.extend(incoming_connections.read());

            if streams.len() >= expected {
                return streams;
            }

            assert!(
                Instant::now() < deadline,
                "initialize_listener did not enqueue the expected number of accepted streams in \
                 time"
            );

            thread::sleep(Duration::from_millis(10));
        }
    }
}
