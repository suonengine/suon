use bevy::prelude::*;
use suon_protocol::packets::client::prelude::*;

use crate::server::{
    connection::Connection,
    packet::{DecodeError, Packet, incoming::IncomingPacket},
};

macro_rules! dispatch_packet {
    ($commands:expr, $client:expr, $incoming_packet:expr; $( $( $kind:pat_param )|+ => $packet_ty:ty ),* $(,)?) => {
        {
            let incoming_packet = $incoming_packet;
            match incoming_packet.kind {
                $(
                    $( $kind )|+ => 'dispatch: {
                        let kind = incoming_packet.kind;
                        let timestamp = incoming_packet.timestamp;
                        let checksum = incoming_packet.checksum;
                        let mut bytes = incoming_packet.buffer.as_ref();

                        let packet = match <$packet_ty>::decode(kind, &mut bytes) {
                            Ok(packet) => packet,
                            Err(err) => {
                                error!("Failed to decode packet for client {}: {:?}", $client, err);
                                let error = DecodeError::from(err);
                                warn!(
                                    "Failed to dispatch typed packet event for client {} and kind {:?}: {}",
                                    $client, kind, error
                                );
                                break 'dispatch;
                            }
                        };

                        if !bytes.is_empty() {
                            let error = DecodeError::ExtraBytes(bytes.len());
                            warn!(
                                "Failed to dispatch typed packet event for client {} and kind {:?}: {}",
                                $client, kind, error
                            );
                            break 'dispatch;
                        }

                        debug!("Successfully decoded packet for client {}", $client);

                        $commands.trigger(Packet {
                            entity: $client,
                            timestamp,
                            checksum,
                            packet,
                        });
                    }
                )*
                _ => {
                    trace!(
                        "Skipping typed dispatch for client packet kind {:?} until a typed event \
                        is registered",
                        incoming_packet.kind
                    );
                }
            }
        }
    };
}

/// Processes all packets received from clients and dispatches typed packet events.
pub(crate) fn process_incoming_client_packets(
    query: Query<(Entity, &Connection)>,
    mut commands: Commands,
    mut incoming_packets: Local<Vec<(Entity, IncomingPacket)>>,
) {
    for (client, connection) in &query {
        for incoming_packet in connection.read() {
            incoming_packets.push((client, incoming_packet));
        }
    }

    incoming_packets.sort_by_key(|(_, packet)| packet.timestamp);

    for (client, incoming_packet) in incoming_packets.drain(..) {
        trace!(
            "Dispatching packet from client {:?}: kind={:?}",
            client, incoming_packet.kind,
        );

        dispatch_packet!(
            commands,
            client,
            incoming_packet;
            PacketKind::AcceptMarketOffer => AcceptMarketOffer,
            PacketKind::AcceptTrade => AcceptTradeOffer,
            PacketKind::AimAtTarget => AimAtTarget,
            PacketKind::AddImbuement => AddImbuement,
            PacketKind::BrowseCharacterInfo => Character,
            PacketKind::BrowseField => Tile,
            PacketKind::BrowseForgeHistory => ForgeHistory,
            PacketKind::BrowseMarket => Market,
            PacketKind::BrowseStoreOffers => BrowseStoreOffers,
            PacketKind::BrowseTransactionHistory => TransactionHistory,
            PacketKind::BuddyGroupAction => BuddyGroupAction,
            PacketKind::BugReport => BugReport,
            PacketKind::BuyCharmRune => BuyCharmRune,
            PacketKind::BuyStoreOffer => BuyStoreOffer,
            PacketKind::CancelMarketOffer => CancelMarketOffer,
            PacketKind::CancelRuleViolation => CancelRuleViolation,
            PacketKind::CancelSteps => CancelSteps,
            PacketKind::CancelTargetAndTrail => CancelTargetAndTrail,
            PacketKind::ChangeMapAwareRange => ChangeMapAwareRange,
            PacketKind::ChangePodium => ChangePodium,
            PacketKind::ChangeSharedPartyExperience => ChangeSharedPartyExperience,
            PacketKind::Channels => Channels,
            PacketKind::ClearImbuement => ClearImbuement,
            PacketKind::CloseContainer => CloseContainer,
            PacketKind::CloseImbuingWindow => CloseImbuingWindow,
            PacketKind::CloseRuleViolation => CloseRuleViolation,
            PacketKind::CloseTrade => CloseTrade,
            PacketKind::CollectRewardChest => CollectRewardChest,
            PacketKind::ConfigureBossSlot => ConfigureBossSlot,
            PacketKind::CreateBuddy => CreateBuddy,
            PacketKind::CreateMarketOffer => CreateMarketOffer,
            PacketKind::CreatePrivateChannel => CreatePrivateChannel,
            PacketKind::CyclopediaHouseAuction => CyclopediaHouseAuction,
            PacketKind::DeleteBuddy => DeleteBuddy,
            PacketKind::DisbandParty => DisbandParty,
            PacketKind::EnterGame => EnterGame,
            PacketKind::EquipItem => EquipItem,
            PacketKind::ExivaRestrictions => ExivaRestrictions,
            PacketKind::ExtendedOpcode => ExtendedOpcode,
            PacketKind::FaceNorth | PacketKind::FaceEast
            | PacketKind::FaceSouth | PacketKind::FaceWest => Face,
            PacketKind::ForgeAction => ForgeAction,
            PacketKind::FriendSystemAction => FriendSystemAction,
            PacketKind::GetRewardDaily => GetRewardDaily,
            PacketKind::InspectItemDetails => InspectItemDetails,
            PacketKind::InspectObject => InspectObject,
            PacketKind::InspectOffer => InspectOffer,
            PacketKind::InspectTrade => InspectTrade,
            PacketKind::InvitePrivateChannel => InvitePrivateChannel,
            PacketKind::InviteToParty => InviteToParty,
            PacketKind::InviteToPrivateChannel => InviteToPrivateChannel,
            PacketKind::JoinChannel => JoinChannel,
            PacketKind::JoinParty => JoinParty,
            PacketKind::KeepAlive => KeepAlive,
            PacketKind::LeaderFinderAction => LeaderFinderAction,
            PacketKind::LeaveChannel => LeaveChannel,
            PacketKind::LeaveMarket => LeaveMarket,
            PacketKind::LeaveNpcChannel => LeaveNpcChannel,
            PacketKind::LeaveNpcShop => LeaveNpcShop,
            PacketKind::LeaveParty => LeaveParty,
            PacketKind::Login => Login,
            PacketKind::Logout => Logout,
            PacketKind::LookAt => LookAt,
            PacketKind::LookInBattleList => LookInBattleList,
            PacketKind::LookInNpcShop => LookInNpcShop,
            PacketKind::LootContainer => LootContainer,
            PacketKind::MemberFinderAction => MemberFinderAction,
            PacketKind::ModalWindowAnswer => ModalWindowAnswer,
            PacketKind::MoveUpContainer => MoveUpContainer,
            PacketKind::OfferTrade => OfferTrade,
            PacketKind::OpenBestiary => OpenBestiary,
            PacketKind::OpenBestiaryOverview => OpenBestiaryOverview,
            PacketKind::OpenBlessDialog => OpenBlessDialog,
            PacketKind::OpenBosstiary => OpenBosstiary,
            PacketKind::OpenOutfitDialog => OpenOutfitDialog,
            PacketKind::OpenParentContainer => OpenParentContainer,
            PacketKind::OpenPreyDialog => OpenPreyDialog,
            PacketKind::OpenQuestLine => OpenQuestLine,
            PacketKind::OpenQuestLog => OpenQuestLog,
            PacketKind::OpenRewardHistory => OpenRewardHistory,
            PacketKind::OpenRewardWall => OpenRewardWall,
            PacketKind::OpenRuleViolation => OpenRuleViolation,
            PacketKind::OpenStore => OpenStore,
            PacketKind::OpenTrackedQuestLog => OpenTrackedQuestLog,
            PacketKind::OpenTransactionHistory => OpenTransactionHistory,
            PacketKind::OpenWheel => OpenWheel,
            PacketKind::PartyAnalyzerAction => PartyAnalyzerAction,
            PacketKind::PassPartyLeadership => PassPartyLeadership,
            PacketKind::PingLatency => PingLatency,
            PacketKind::PreyAction => PreyAction,
            PacketKind::PurchaseNpcShop => PurchaseNpcShop,
            PacketKind::QueryBossSlotInfo => QueryBossSlotInfo,
            PacketKind::QueryDepotSearchItem => QueryDepotSearchItem,
            PacketKind::QueryHighscores => QueryHighscores,
            PacketKind::QuickLoot => QuickLoot,
            PacketKind::QuickLootFilter => QuickLootFilter,
            PacketKind::RemoveFromPrivateChannel => RemoveFromPrivateChannel,
            PacketKind::RetrieveDepotSearch => RetrieveDepotSearch,
            PacketKind::RevokePartyInvite => RevokePartyInvite,
            PacketKind::RotateItem => RotateItem,
            PacketKind::RuleViolationReport => RuleViolationReport,
            PacketKind::SaveWheel => SaveWheel,
            PacketKind::Say => Say,
            PacketKind::SearchBestiary => SearchBestiary,
            PacketKind::SeekInContainer => SeekInContainer,
            PacketKind::SellNpcShop => SellNpcShop,
            PacketKind::ServerName => ServerName,
            PacketKind::SetMonsterPodium => SetMonsterPodium,
            PacketKind::SetMountState => SetMountState,
            PacketKind::SetTypingState => SetTypingState,
            PacketKind::StashAction => StashAction,
            PacketKind::StepNorth | PacketKind::StepEast
            | PacketKind::StepSouth | PacketKind::StepWest
            | PacketKind::StepNorthEast | PacketKind::StepSouthEast
            | PacketKind::StepSouthWest | PacketKind::StepNorthWest => Step,
            PacketKind::Steps => Steps,
            PacketKind::SubmitHouseWindow => SubmitHouseWindow,
            PacketKind::SubmitTextWindow => SubmitTextWindow,
            PacketKind::Target => Target,
            PacketKind::TaskHuntingAction => TaskHuntingAction,
            PacketKind::Teleport => Teleport,
            PacketKind::ThrowItem => ThrowItem,
            PacketKind::Trail => Trail,
            PacketKind::TransferCoins => TransferCoins,
            PacketKind::UpdateBuddy => UpdateBuddy,
            PacketKind::UpdateFightModes => UpdateFightModes,
            PacketKind::UpdateInventoryImbuements => UpdateInventoryImbuements,
            PacketKind::UpdateMonsterTracker => UpdateMonsterTracker,
            PacketKind::UpdateOutfit => UpdateOutfit,
            PacketKind::UseItem => UseItem,
            PacketKind::UseItemWithCreature => UseItemWithCreature,
            PacketKind::UseItemWithTarget => UseItemWithTarget,
            PacketKind::WheelGem => WheelGem,
            PacketKind::WrapItem => Wrap,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{
        connection::Connection,
        packet::{Packet, incoming::IncomingPacket},
        settings::PacketPolicy,
    };
    use bytes::Bytes;
    use std::{
        net::{Ipv4Addr, SocketAddr, SocketAddrV4},
        time::Instant,
    };
    use suon_checksum::Adler32Checksum;
    use suon_protocol::packets::client::DecodableError;

    #[derive(Resource, Default, Debug, PartialEq, Eq)]
    struct ObservedPackets(Vec<&'static str>);

    #[derive(Resource, Default, Debug, PartialEq, Eq)]
    struct PingLatencyMeta(Option<(Entity, Instant, Option<Adler32Checksum>)>);

    #[derive(Resource, Default, Debug, PartialEq, Eq)]
    struct MoveDirections(Vec<suon_position::direction::Direction>);

    #[derive(Debug)]
    struct Failing;

    impl Decodable for Failing {
        fn decode(_: PacketKind, _: &mut &[u8]) -> Result<Self, DecodableError> {
            Err(DecodableError::Decoder(
                suon_protocol::packets::decoder::DecoderError::Incomplete {
                    expected: 1,
                    available: 0,
                },
            ))
        }
    }

    fn observe_keep_alive(event: On<Packet<KeepAlive>>, mut observed: ResMut<ObservedPackets>) {
        assert!(
            matches!(event.packet(), KeepAlive),
            "The keep-alive observer should receive the decoded packet payload"
        );
        observed.0.push("keep_alive");
    }

    fn observe_ping_latency(
        event: On<Packet<PingLatency>>,
        mut observed: ResMut<ObservedPackets>,
        mut metadata: ResMut<PingLatencyMeta>,
    ) {
        assert!(
            matches!(event.packet(), PingLatency),
            "The ping-latency observer should receive the decoded packet payload"
        );
        observed.0.push("ping_latency");
        metadata.0 = Some((event.entity(), event.timestamp(), event.checksum()));
    }

    fn observe_step_packet(
        event: On<Packet<Step>>,
        mut observed: ResMut<ObservedPackets>,
        mut directions: ResMut<MoveDirections>,
    ) {
        observed.0.push("step");
        directions.0.push(event.packet().direction);
    }

    fn observe_server_name(event: On<Packet<ServerName>>, mut observed: ResMut<ObservedPackets>) {
        observed.0.push("server_name");
        assert_eq!(
            event.packet().server_name,
            "otserv",
            "The server-name observer should receive the decoded handshake string"
        );
    }

    fn observe_login(event: On<Packet<Login>>, mut observed: ResMut<ObservedPackets>) {
        observed.0.push("login");
        assert_eq!(
            event.packet().payload,
            vec![1, 2, 3],
            "The login observer should receive the preserved raw login payload"
        );
    }

    fn observe_steps_packet(event: On<Packet<Steps>>, mut observed: ResMut<ObservedPackets>) {
        observed.0.push("steps");
        assert!(
            !event.packet().path.is_empty(),
            "Steps observers should receive at least one decoded direction"
        );
    }

    fn build_connection(packets: impl IntoIterator<Item = IncomingPacket>) -> Connection {
        let (outgoing_sender, _outgoing_receiver) = crossbeam_channel::unbounded();
        let (incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        for packet in packets {
            incoming_sender
                .send(packet)
                .expect("The incoming packet channel should accept the test packet");
        }

        Connection::new(
            outgoing_sender,
            incoming_receiver,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172)),
            xtea_sender,
            PacketPolicy::default(),
        )
    }

    fn build_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ObservedPackets>();
        app.init_resource::<PingLatencyMeta>();
        app.init_resource::<MoveDirections>();
        app.add_observer(observe_keep_alive);
        app.add_observer(observe_login);
        app.add_observer(observe_ping_latency);
        app.add_observer(observe_server_name);
        app.add_observer(observe_step_packet);
        app.add_observer(observe_steps_packet);
        app.add_systems(Update, process_incoming_client_packets);
        app
    }

    #[test]
    fn should_dispatch_all_queued_incoming_packets_as_typed_events() {
        let mut app = build_app();

        let timestamp = Instant::now();
        let checksum = Adler32Checksum::from(0x1234_5678_u32);
        let connection = build_connection([IncomingPacket {
            timestamp,
            checksum: Some(checksum),
            kind: PacketKind::PingLatency,
            buffer: Bytes::new(),
        }]);

        let client = app.world_mut().spawn(connection).id();

        app.update();
        app.update();

        let observed = app.world().resource::<ObservedPackets>();
        let metadata = app.world().resource::<PingLatencyMeta>();

        assert_eq!(
            observed.0,
            vec!["ping_latency"],
            "process_incoming_client_packets should emit one typed event per queued incoming \
             packet"
        );
        assert_eq!(
            metadata.0,
            Some((client, timestamp, Some(checksum))),
            "typed packet dispatch should preserve the original packet metadata"
        );
    }

    #[test]
    fn should_not_emit_typed_events_when_connections_have_no_incoming_packets() {
        let mut app = build_app();

        let (outgoing_sender, _outgoing_receiver) = crossbeam_channel::unbounded();
        let (_incoming_sender, incoming_receiver) = crossbeam_channel::unbounded();
        let (xtea_sender, _xtea_receiver) = tokio::sync::watch::channel(None);

        let connection = Connection::new(
            outgoing_sender,
            incoming_receiver,
            SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 7172)),
            xtea_sender,
            PacketPolicy::default(),
        );

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        let messages = app.world().resource::<ObservedPackets>();
        assert_eq!(
            messages.0.len(),
            0,
            "process_incoming_client_packets should skip connections without queued packets"
        );
    }

    #[test]
    fn should_dispatch_every_packet_currently_queued_on_the_connection_in_order() {
        let mut app = build_app();

        let connection = build_connection([
            IncomingPacket {
                timestamp: Instant::now(),
                checksum: None,
                kind: PacketKind::KeepAlive,
                buffer: Bytes::from_static(&[]),
            },
            IncomingPacket {
                timestamp: Instant::now(),
                checksum: None,
                kind: PacketKind::PingLatency,
                buffer: Bytes::new(),
            },
            IncomingPacket {
                timestamp: Instant::now(),
                checksum: None,
                kind: PacketKind::StepNorthWest,
                buffer: Bytes::new(),
            },
            IncomingPacket {
                timestamp: Instant::now(),
                checksum: None,
                kind: PacketKind::Steps,
                buffer: Bytes::from_static(&[2, 1, 3]),
            },
        ]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        let observed = app.world().resource::<ObservedPackets>();

        assert_eq!(
            observed.0,
            vec!["keep_alive", "ping_latency", "step", "steps"],
            "process_incoming_client_packets should dispatch all packets currently queued in \
             receive order"
        );
    }

    #[test]
    fn should_dispatch_server_name_packets_when_a_typed_packet_is_registered() {
        let mut app = build_app();

        let connection = build_connection([IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::ServerName,
            buffer: Bytes::from_static(&[6, 0, b'o', b't', b's', b'e', b'r', b'v']),
        }]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<ObservedPackets>().0,
            vec!["server_name"],
            "ServerName packets should dispatch once a typed packet is registered"
        );
    }

    #[test]
    fn should_ignore_kinds_without_registered_typed_dispatch() {
        let mut app = build_app();

        let connection = build_connection([IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::OpenStore,
            buffer: Bytes::new(),
        }]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        assert!(
            app.world().resource::<ObservedPackets>().0.is_empty(),
            "Unregistered packet kinds should not emit typed packet events"
        );
    }

    #[test]
    fn should_dispatch_login_packets_when_a_typed_packet_is_registered() {
        let mut app = build_app();

        let connection = build_connection([IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::Login,
            buffer: Bytes::from_static(&[1, 2, 3]),
        }]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        assert_eq!(
            app.world().resource::<ObservedPackets>().0,
            vec!["login"],
            "Login packets should dispatch once a typed packet is registered"
        );
    }

    #[test]
    fn should_not_emit_typed_events_when_payload_has_extra_bytes() {
        let mut app = build_app();

        let connection = build_connection([IncomingPacket {
            timestamp: Instant::now(),
            checksum: None,
            kind: PacketKind::KeepAlive,
            buffer: Bytes::from_static(&[1]),
        }]);

        app.world_mut().spawn(connection);
        app.update();
        app.update();

        assert!(
            app.world().resource::<ObservedPackets>().0.is_empty(),
            "Packets that leave unread payload bytes should be rejected during typed dispatch"
        );
    }

    #[test]
    fn should_not_emit_typed_events_when_decoding_fails() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, |mut commands: Commands| {
            dispatch_packet!(
                commands,
                Entity::from_bits(7),
                IncomingPacket {
                    timestamp: Instant::now(),
                    checksum: None,
                    kind: PacketKind::PingLatency,
                    buffer: Bytes::new(),
                };
                PacketKind::PingLatency => Failing,
            );
        });

        #[derive(Resource, Default)]
        struct Triggered(bool);

        fn observe_failing_packet(_event: On<Packet<Failing>>, mut triggered: ResMut<Triggered>) {
            triggered.0 = true;
        }

        app.init_resource::<Triggered>();
        app.add_observer(observe_failing_packet);

        app.update();
        app.update();

        assert!(
            !app.world().resource::<Triggered>().0,
            "Failed packet decodes should not emit typed packet events"
        );
    }
}
