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
    => b"csm" + clientID + magic + message.len() + message;
Server Broadcast Message            (sbm):
    => b"sbm" + username.len() + username +message.len() + message;
Heart Beat Request                  (hbr):
    => b"hbr"
Heart Beat Send                     (hbs):
    => b"hbs" + clientID + magic
*/

mod parse {
    use super::*;
    #[test]
    fn ClientSendMessage() {
        assert_eq!(
            ClientSendMessage::from_bytes(b"csm\x00\x00\x00\xFF\x00\x00\xFF\x00\x00\x06AZERTY")
                .unwrap()
                .1,
            ClientSendMessage {
                client_id: 0x000000FF,
                magic: 0x0000FF00,
                message_len: 6,
                message: "AZERTY"
            }
        )
    }

    #[test]
    fn ClientRegistrationRequest() {
        assert_eq!(
            ClientRegistrationRequest::from_bytes(b"crr\x04Maix")
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
            HeartBeatSend::from_bytes(b"hbs\xFF\xAA\xFF\xAA\x12\x34\x56\x78")
                .unwrap()
                .1,
            HeartBeatSend {
                client_id: 0xFFAAFFAA,
                magic: 0x12345678
            }
        )
    }
    #[test]
    fn ServerRegistrationConfirmation() {
        assert_eq!(
            ServerRegistrationConfirmation::from_bytes(b"src\x00\x00\x00\xFF\x00\x00\xFF\x00")
                .unwrap()
                .1,
            ServerRegistrationConfirmation {
                client_id: 0x000000FF,
                magic: 0x0000FF00,
            }
        )
    }

    #[test]
    fn ServerBroadcastMessage() {
        assert_eq!(
            ServerBroadcastMessage::from_bytes(
                b"sbm\xFF\xDD\x00\xFF\x04Maix\x00\x0FJeSuisUneBanane"
            )
            .unwrap()
            .1,
            ServerBroadcastMessage {
                user_id: 0xFFDD00FF,
                username_len: 0x04,
                username: "Maix",
                message_len: 0x0F,
                message: "JeSuisUneBanane"
            }
        )
    }

    #[test]
    fn HeartBeatRequest() {
        assert_eq!(
            HeartBeatRequest::from_bytes(b"hbr").unwrap().1,
            HeartBeatRequest {}
        )
    }
}

#[cfg(test)]
mod serialize {
    use super::*;
    #[test]
    fn ClientSendMessage() {
        assert_eq!(
            ClientSendMessage {
                client_id: 0x000000FF,
                magic: 0x0000FF00,
                message_len: 6,
                message: "AZERTY"
            }
            .unwrap_bytes(),
            b"csm\x00\x00\x00\xFF\x00\x00\xFF\x00\x00\x06AZERTY"
        )
    }
    #[test]
    fn ClientRegistrationRequest() {
        assert_eq!(
            ClientRegistrationRequest {
                username_len: 4,
                username: "Maix"
            }
            .unwrap_bytes(),
            b"crr\x04Maix"
        )
    }

    #[test]
    fn HeartBeatSend() {
        assert_eq!(
            HeartBeatSend {
                client_id: 0xFFAAFFAA,
                magic: 0x12345678
            }
            .unwrap_bytes(),
            b"hbs\xFF\xAA\xFF\xAA\x12\x34\x56\x78"
        )
    }

    #[test]
    fn ServerRegistrationConfirmation() {
        assert_eq!(
            ServerRegistrationConfirmation {
                client_id: 0x000000FF,
                magic: 0x0000FF00,
            }
            .unwrap_bytes(),
            b"src\x00\x00\x00\xFF\x00\x00\xFF\x00"
        )
    }

    #[test]
    fn ServerBroadcastMessage() {
        assert_eq!(
            ServerBroadcastMessage {
                user_id: 0xFFDD00FF,
                username_len: 0x04,
                username: "Maix",
                message_len: 0x0F,
                message: "JeSuisUneBanane"
            }
            .unwrap_bytes(),
            b"sbm\xFF\xDD\x00\xFF\x04Maix\x00\x0FJeSuisUneBanane"
        )
    }

    #[test]
    fn HeartBeatRequest() {
        assert_eq!(HeartBeatRequest {}.unwrap_bytes(), b"hbr")
    }
}
