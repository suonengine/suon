pub enum Command {
    /// Encrypt and frame the data using the current protocol settings.
    Send(Vec<u8>),
    /// Send raw bytes without any framing or encryption.
    SendRaw(Vec<u8>),
    /// Replace the XTEA encryption key.
    SetXteaKey([u32; 4]),
    /// Enable or disable XTEA encryption.
    SetEncryptionEnabled(bool),
    /// Change the minimum payload size that triggers compression.
    SetCompressionThreshold(usize),
    /// Close the connection gracefully.
    Close,
    /// Close the connection with a human-readable reason.
    CloseWithReason(String),
}

pub(crate) type CommandSender = crossbeam_channel::Sender<Command>;
#[allow(dead_code)]
pub(crate) type CommandReceiver = crossbeam_channel::Receiver<Command>;

#[cfg(test)]
mod tests {
    use crate::protocol::command::*;

    #[test]
    fn command_send_holds_data() {
        let data = vec![0x01, 0x02, 0x03];
        let cmd = Command::Send(data.clone());
        match cmd {
            Command::Send(d) => assert_eq!(d, data),
            _ => panic!("expected Send"),
        }
    }

    #[test]
    fn command_close_no_data() {
        let cmd = Command::Close;
        assert!(matches!(cmd, Command::Close));
    }

    #[test]
    fn command_sender_receiver_roundtrip() {
        let (tx, rx) = crossbeam_channel::bounded(16);
        tx.send(Command::Send(vec![42]))
            .expect("failed to send Send command in roundtrip test");
        tx.send(Command::Close)
            .expect("failed to send Close command in roundtrip test");

        assert!(
            matches!(rx.recv().expect("failed to receive Send command"), Command::Send(d) if d == vec![42])
        );
        assert!(matches!(
            rx.recv().expect("failed to receive Close command"),
            Command::Close
        ));
    }

    #[test]
    fn command_sender_bounded_backpressure() {
        let (tx, rx) = crossbeam_channel::bounded(2);
        tx.send(Command::Send(vec![1]))
            .expect("failed to send first command in backpressure test");
        tx.send(Command::Send(vec![2]))
            .expect("failed to send second command in backpressure test");
        assert!(tx.try_send(Command::Send(vec![3])).is_err());
        drop(rx);
    }

    #[test]
    fn command_set_xtea_key() {
        let (tx, rx) = crossbeam_channel::bounded(16);
        let key = [0x01, 0x23, 0x45, 0x67];
        tx.send(Command::SetXteaKey(key))
            .expect("failed to send SetXteaKey command");
        assert!(
            matches!(rx.recv().expect("failed to receive SetXteaKey command"), Command::SetXteaKey(k) if k == key)
        );
    }

    #[test]
    fn command_close_with_reason() {
        let (tx, rx) = crossbeam_channel::bounded(16);
        tx.send(Command::CloseWithReason("shutdown".into()))
            .expect("failed to send CloseWithReason command");
        assert!(
            matches!(rx.recv().expect("failed to receive CloseWithReason command"), Command::CloseWithReason(r) if r == "shutdown")
        );
    }
}
