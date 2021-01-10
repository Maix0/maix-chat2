#![allow(non_snake_case)]
use crate::parser::FromBytes;
use crate::serializer::IntoBytes;
use crate::*;

/*
Client Registration Request         (crr):
    => b"crr" + username.len() + username;
Server Registration Confirmation    (src):
    => b"src" + clientID + Magic;
Client Send Message                 (csm):
    => b"csm" + clientID + solved + message.len() + message;
Server Broadcast Message            (sbm):
    => b"sbm" + new_magic + username.len() + username +message.len() + message;
Heart Beat Request                  (hbr):
    => b"hbr" + new_magic
Heart Beat Send                     (hbs):
    => b"hbs" + clientID + solved
*/
mod parse {
    use super::*;
    #[test]
    fn ClientSendMessage() {
        assert_eq!(
            ClientSendMessage::from_bytes(b"\x00\x00\x00\xFF\x00\x00\xFF\x00\x00\x06AZERTY")
                .unwrap()
                .1,
            ClientSendMessage {
                client_id: 0x000000FF,
                solved: 0x0000FF00,
                message_len: 6,
                message: "AZERTY"
            }
        )
    }

    #[test]
    fn ClientRegistrationRequest() {
        assert_eq!(
            ClientRegistrationRequest::from_bytes(b"\x04Maix")
                .unwrap()
                .1,
            ClientRegistrationRequest {
                username_len: 4,
                username: "Maix"
            }
        )
    }
    #[test]
    fn HeartBeatSend() {
        assert_eq!(
            HeartBeatSend::from_bytes(b"\xFF\xAA\xFF\xAA\x12\x34\x56\x78")
                .unwrap()
                .1,
            HeartBeatSend {
                client_id: 0xFFAAFFAA,
                solved: 0x12345678
            }
        )
    }
}
