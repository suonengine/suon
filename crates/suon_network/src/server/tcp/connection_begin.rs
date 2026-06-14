use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use suon_channel::TaskHandler;
use suon_lua::LuaVm;
use suon_macros::Task;
use suon_resource::Resources;

use crate::connection::id::ConnectionId;

/// Writes an IP address into a fixed stack buffer.
fn fmt_ip(ip: IpAddr, buf: &mut [u8; 48]) -> &str {
    match ip {
        IpAddr::V4(v4) => fmt_ipv4(v4, buf),
        IpAddr::V6(v6) => fmt_ipv6(v6, buf),
    }
}

fn fmt_ipv4(ip: Ipv4Addr, buf: &mut [u8; 48]) -> &str {
    let octets = ip.octets();
    let mut pos = 0;
    for (i, &octet) in octets.iter().enumerate() {
        if i > 0 {
            buf[pos] = b'.';
            pos += 1;
        }
        let n = octet;
        if n >= 100 {
            buf[pos] = b'0' + n / 100;
            buf[pos + 1] = b'0' + (n / 10) % 10;
            buf[pos + 2] = b'0' + n % 10;
            pos += 3;
        } else if n >= 10 {
            buf[pos] = b'0' + n / 10;
            buf[pos + 1] = b'0' + n % 10;
            pos += 2;
        } else {
            buf[pos] = b'0' + n;
            pos += 1;
        }
    }
    // SAFETY: we only write ASCII digits and dots
    unsafe { std::str::from_utf8_unchecked(&buf[..pos]) }
}

fn fmt_ipv6(ip: Ipv6Addr, buf: &mut [u8; 48]) -> &str {
    let segments = ip.segments();
    let mut pos = 0;
    for (i, &seg) in segments.iter().enumerate() {
        if i > 0 {
            buf[pos] = b':';
            pos += 1;
        }
        // Write hex digits (no leading zeros for brevity, like Rust's Display)
        if seg == 0 {
            buf[pos] = b'0';
            pos += 1;
        } else {
            let mut started = false;
            for shift in (0..16).step_by(4).rev() {
                let digit = (seg >> shift) as u8 & 0xF;
                if digit != 0 || started {
                    buf[pos] = hex_char(digit);
                    pos += 1;
                    started = true;
                }
            }
        }
    }
    unsafe { std::str::from_utf8_unchecked(&buf[..pos]) }
}

fn hex_char(d: u8) -> u8 {
    if d < 10 { b'0' + d } else { b'a' + d - 10 }
}

/// Task sent from the Tokio accept loop to the main thread when a new
/// TCP connection arrives.
///
/// When `response` is `Some(sender)` the main thread will wait for a Lua
/// `onConnect` handler to accept or reject the connection before the
/// reader/writer sessions are spawned.  When `None` (e.g. in tests) the
/// connection is always accepted.
#[derive(Task)]
pub(crate) struct ConnectionBegin {
    pub id: ConnectionId,
    pub address: SocketAddr,
    pub response: Option<tokio::sync::oneshot::Sender<bool>>,
}

impl TaskHandler for ConnectionBegin {
    fn run(&mut self, resources: &mut Resources) {
        let lua_vm = resources.get::<LuaVm>();
        let mut ip_buf = [0u8; 48];
        let ip_str = fmt_ip(self.address.ip(), &mut ip_buf);
        let accepted = lua_vm
            .trigger_event(
                "ConnectionBeginEvent",
                (self.id.as_u64(), ip_str, self.address.port()),
            )
            .is_ok();

        if let Some(sender) = self.response.take() {
            if let Err(error) = sender.send(accepted) {
                tracing::error!(
                    target: "TCP",
                    "Failed to send ConnectionBegin response for {}: {error}",
                    self.id,
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn connection_begin_fields() {
        let task = ConnectionBegin {
            id: ConnectionId::new(0, 42),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 7171),
            response: None,
        };
        assert_eq!(task.id.sequence(), 42);
        assert_eq!(task.address.port(), 7171);
    }

    #[test]
    fn connection_begin_task_run_does_not_panic() {
        let mut resources = suon_resource::Resources::default();
        resources.insert(suon_lua::LuaVm::new());
        resources.insert(suon_channel::Channel::default());
        let mut task = Box::new(ConnectionBegin {
            id: ConnectionId::new(0, 1),
            address: SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7171),
            response: None,
        });
        task.run(&mut resources);
    }
}
