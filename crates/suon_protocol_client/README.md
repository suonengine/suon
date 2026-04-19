# suon_protocol_client

Client-to-server packet definitions for the Suon MMORPG framework.

`suon_protocol_client` provides:

- Typed structs for every packet a game client can send
- The `Decodable` trait for decoding raw bytes into typed packets
- The `PacketKind` trait mapping packets to their 1-byte discriminant
- A `DecodableError` type for decoding failures

## Installation

```toml
[dependencies]
suon_protocol_client = { path = "../suon_protocol_client" }
suon_protocol = { path = "../suon_protocol" }
```

## Quick Start

```rust,ignore
use suon_protocol_client::prelude::*;

fn dispatch(kind: u8, buf: &[u8]) {
    match kind {
        StepPacket::KIND => {
            let packet = StepPacket::decode(buf).expect("invalid step packet");
            println!("Step direction: {:?}", packet.direction);
        }
        KeepAlivePacket::KIND => {
            println!("Keep-alive received");
        }
        _ => eprintln!("Unknown packet kind: {kind:#04x}"),
    }
}
```

## How It Works

Each packet struct derives `Decodable` and implements `PacketKind`. The 1-byte kind
discriminant is defined by the MMORPG protocol and is used to route raw network bytes
to the correct struct before deserialization.

## Packet Reference

### Movement

| Packet | Description |
|---|---|
| `StepPacket` | Single-tile step in a direction |
| `StepsPacket` | Sequence of tile steps (path following) |
| `CancelStepsPacket` | Cancel the current step sequence |
| `FacePacket` | Change facing direction without moving |

### Social

| Packet | Description |
|---|---|
| `CreateBuddyPacket` | Add a character to the buddy list |
| `DeleteBuddyPacket` | Remove a character from the buddy list |
| `UpdateBuddyPacket` | Update buddy list entry |

### Party

| Packet | Description |
|---|---|
| `InviteToPartyPacket` | Invite a player to the party |
| `JoinPartyPacket` | Accept a party invitation |
| `LeavePartyPacket` | Leave the current party |
| `PassPartyLeadershipPacket` | Transfer party leadership |
| `RevokePartyInvitePacket` | Cancel an outgoing party invitation |
| `ChangeSharedPartyExperiencePacket` | Toggle shared experience within the party |

### Trade

| Packet | Description |
|---|---|
| `RequestTradePacket` | Initiate a trade with another player |
| `AcceptTradePacket` | Accept an incoming trade request |
| `InspectTradePacket` | View the current trade window |
| `CloseTradePacket` | Close and cancel the trade |

### Market

| Packet | Description |
|---|---|
| `BrowseMarketPacket` | Browse market listings |
| `CreateMarketOfferPacket` | Post a buy or sell offer |
| `AcceptMarketOfferPacket` | Accept an existing market offer |
| `CancelMarketOfferPacket` | Cancel one of your market offers |
| `LeaveMarketPacket` | Close the market window |
| `MarketOfferKind` | Enum: `Buy` or `Sell` |

### Connection

| Packet | Description |
|---|---|
| `KeepAlivePacket` | Heartbeat to keep the connection alive |
| `PingLatencyPacket` | Round-trip latency measurement |
