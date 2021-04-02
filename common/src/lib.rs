#[macro_use]
extern crate thiserror;

pub mod parser;
pub mod serializer;

#[cfg(test)]
mod tests;
/*
Client Registration Request         (crr):
    => b"crr" + username.len() + username;
Server Registration Confirmation    (src):
    => b"src" + clientID + Magic;
ClientRegistrationEnd               (cre)
    => b"cre" + clientID + magic
Client Send Message                 (csm):
    => b"csm" + clientID + magic + message.len() + message;
Server Broadcast Message            (sbm):
    => b"sbm" + username.len() + username +message.len() + message;
Heart Beat Request                  (hbr):
    => b"hbr"
Heart Beat Send                     (hbs):
    => b"hbs" + clientID + magic
*/

#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum Packet<'a> {
    ClientRegistrationRequest(ClientRegistrationRequest<'a>),
    ClientRegistrationEnd(ClientRegistrationEnd),
    ClientSendMessage(ClientSendMessage<'a>),
    HeartBeatSend(HeartBeatSend),

    ServerRegistrationConfirmation(ServerRegistrationConfirmation),
    ServerBroadcastMessage(ServerBroadcastMessage<'a>),
    HeartBeatRequest(HeartBeatRequest),
}

impl<'a> Packet<'a> {
    pub fn into_owned(&self) -> PacketOwned {
        macro_rules! into_owned {
            (($($changed:ident),*),($($owned:ident),*)) => {
                match self {
                    $(Packet::$owned(inner) => PacketOwned::$owned(*inner),)*
                    $(Packet::$changed(inner) => PacketOwned::$changed(inner.into_owned()),)*
                }
            };
        }

        into_owned!(
            (
                ClientRegistrationRequest,
                ClientSendMessage,
                ServerBroadcastMessage
            ),
            (
                ClientRegistrationEnd,
                HeartBeatSend,
                HeartBeatRequest,
                ServerRegistrationConfirmation
            )
        )
    }
}

impl<'a> ClientRegistrationRequest<'a> {
    pub fn into_owned(&self) -> ClientRegistrationRequestOwned {
        ClientRegistrationRequestOwned {
            username_len: self.username_len,
            username: self.username.to_owned(),
        }
    }
}

impl<'a> ClientSendMessage<'a> {
    pub fn into_owned(&self) -> ClientSendMessageOwned {
        ClientSendMessageOwned {
            client_id: self.client_id,
            magic: self.magic,
            message_len: self.message_len,
            message: self.message.to_owned(),
        }
    }
}

impl<'a> ServerBroadcastMessage<'a> {
    pub fn into_owned(&self) -> ServerBroadcastMessageOwned {
        ServerBroadcastMessageOwned {
            user_id: self.user_id,
            username_len: self.username_len,
            username: self.username.to_owned(),
            message_len: self.message_len,
            message: self.message.to_owned(),
        }
    }
}
#[derive(Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum PacketOwned {
    ClientRegistrationRequest(ClientRegistrationRequestOwned),
    ClientRegistrationEnd(ClientRegistrationEnd),
    ClientSendMessage(ClientSendMessageOwned),
    HeartBeatSend(HeartBeatSend),

    ServerRegistrationConfirmation(ServerRegistrationConfirmation),
    ServerBroadcastMessage(ServerBroadcastMessageOwned),
    HeartBeatRequest(HeartBeatRequest),
}

impl<'a> Packet<'a> {
    pub fn get_identifier(&self) -> [u8; 3] {
        use Packet::*;
        match self {
            ClientRegistrationRequest(inner) => inner.get_identifier(),
            ClientRegistrationEnd(inner) => inner.get_identifier(),
            ClientSendMessage(inner) => inner.get_identifier(),
            HeartBeatSend(inner) => inner.get_identifier(),

            ServerBroadcastMessage(inner) => inner.get_identifier(),
            ServerRegistrationConfirmation(inner) => inner.get_identifier(),
            HeartBeatRequest(inner) => inner.get_identifier(),
        }
    }
}

impl PacketOwned {
    pub fn get_identifier(&self) -> [u8; 3] {
        use PacketOwned::*;
        match self {
            ClientRegistrationRequest(inner) => inner.get_identifier(),
            ClientRegistrationEnd(inner) => inner.get_identifier(),
            ClientSendMessage(inner) => inner.get_identifier(),
            HeartBeatSend(inner) => inner.get_identifier(),

            ServerBroadcastMessage(inner) => inner.get_identifier(),
            ServerRegistrationConfirmation(inner) => inner.get_identifier(),
            HeartBeatRequest(inner) => inner.get_identifier(),
        }
    }
}

impl ClientRegistrationEnd {
    const IDENTIFIER: [u8; 3] = *b"cre";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}

impl<'a> ClientRegistrationRequest<'a> {
    const IDENTIFIER: [u8; 3] = *b"crr";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}
impl<'a> ClientSendMessage<'a> {
    const IDENTIFIER: [u8; 3] = *b"csm";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}

impl ClientRegistrationRequestOwned {
    const IDENTIFIER: [u8; 3] = *b"crr";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}
impl ClientSendMessageOwned {
    const IDENTIFIER: [u8; 3] = *b"csm";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}
impl HeartBeatSend {
    const IDENTIFIER: [u8; 3] = *b"hbs";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}
impl<'a> ServerBroadcastMessage<'a> {
    const IDENTIFIER: [u8; 3] = *b"sbm";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}

impl ServerBroadcastMessageOwned {
    const IDENTIFIER: [u8; 3] = *b"sbm";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}
impl ServerRegistrationConfirmation {
    const IDENTIFIER: [u8; 3] = *b"src";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}
impl HeartBeatRequest {
    const IDENTIFIER: [u8; 3] = *b"hbr";
    pub fn get_identifier(&self) -> [u8; 3] {
        Self::IDENTIFIER
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ClientRegistrationEnd {
    pub client_id: u32,
    pub magic: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientRegistrationRequest<'a> {
    pub username_len: u8,
    pub username: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientRegistrationRequestOwned {
    pub username_len: u8,
    pub username: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ServerRegistrationConfirmation {
    pub client_id: u32,
    pub magic: u32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientSendMessage<'a> {
    pub client_id: u32,
    pub magic: u32,
    pub message_len: u16,
    pub message: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ClientSendMessageOwned {
    pub client_id: u32,
    pub magic: u32,
    pub message_len: u16,
    pub message: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServerBroadcastMessage<'a> {
    pub user_id: u32,
    pub username_len: u8,
    pub username: &'a str,
    pub message_len: u16,
    pub message: &'a str,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ServerBroadcastMessageOwned {
    pub user_id: u32,
    pub username_len: u8,
    pub username: String,
    pub message_len: u16,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HeartBeatRequest {}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HeartBeatSend {
    pub client_id: u32,
    pub magic: u32,
}
