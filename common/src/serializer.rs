extern crate cookie_factory as cookie;

use crate::{
    ClientRegistrationRequest, ClientSendMessage, HeartBeatRequest, HeartBeatSend, Packet,
    ServerBroadcastMessage, ServerRegistrationConfirmation,
};
pub trait IntoBytes {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>>;
    fn into_bytes_unwrap(&self) -> Vec<u8> {
        self.into_bytes().unwrap().into_inner().0
    }
}

macro_rules! packet_def {
    ($input:expr, ($($packet:ident),*) ) => {
        match $input {
            $(Packet::$packet(p) => p.into_bytes(),)*
        }
    };
}

impl<'a> IntoBytes for Packet<'a> {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>> {
        packet_def!(
            self,
            (
                ClientSendMessage,
                ClientRegistrationRequest,
                HeartBeatSend,
                HeartBeatRequest,
                ServerBroadcastMessage,
                ServerRegistrationConfirmation
            )
        )
    }
}

impl<'a> IntoBytes for ClientSendMessage<'a> {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>> {
        let raw_buffer: Vec<u8> = vec![0; 3 + 4 + 4 + 2 + self.message.bytes().len()];
        let context = cookie::WriteContext::from(raw_buffer);
        let context = cookie::combinator::slice(&Self::IDENTIFIER)(context)?;

        let context = cookie::bytes::be_u32(self.client_id)(context)?;
        let context = cookie::bytes::be_u32(self.solved)(context)?;
        let context = cookie::bytes::be_u16(self.message_len)(context)?;
        let context = cookie::combinator::string(&self.message)(context)?;

        Ok(context)
    }
}
impl<'a> IntoBytes for ClientRegistrationRequest<'a> {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>> {
        let raw_buffer: Vec<u8> = vec![0; 3 + 1 + self.username.bytes().len()];
        let context = cookie::WriteContext::from(raw_buffer);
        let context = cookie::combinator::slice(&Self::IDENTIFIER)(context)?;

        let context = cookie::bytes::be_u8(self.username_len)(context)?;
        let context = cookie::combinator::string(&self.username)(context)?;

        Ok(context)
    }
}
impl IntoBytes for HeartBeatSend {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>> {
        let raw_buffer: Vec<u8> = vec![0; 3 + 4 + 4];
        let context = cookie::WriteContext::from(raw_buffer);
        let context = cookie::combinator::slice(&Self::IDENTIFIER)(context)?;

        let context = cookie::bytes::be_u32(self.client_id)(context)?;
        let context = cookie::bytes::be_u32(self.solved)(context)?;

        Ok(context)
    }
}

impl IntoBytes for HeartBeatRequest {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>> {
        let raw_buffer: Vec<u8> = vec![0; 3 + 4];
        let context = cookie::WriteContext::from(raw_buffer);
        let context = cookie::combinator::slice(&Self::IDENTIFIER)(context)?;

        let context = cookie::bytes::be_u32(self.new_magic)(context)?;

        Ok(context)
    }
}
impl IntoBytes for ServerRegistrationConfirmation {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>> {
        let raw_buffer: Vec<u8> = vec![0; 3 + 4];
        let context = cookie::WriteContext::from(raw_buffer);
        let context = cookie::combinator::slice(&Self::IDENTIFIER)(context)?;

        let context = cookie::bytes::be_u32(self.client_id)(context)?;
        let context = cookie::bytes::be_u32(self.magic)(context)?;

        Ok(context)
    }
}
impl<'a> IntoBytes for ServerBroadcastMessage<'a> {
    fn into_bytes(&self) -> cookie::GenResult<Vec<u8>> {
        let raw_buffer: Vec<u8> =
            vec![0; 3 + 4 + 4 + 1 + self.username.bytes().len() + 2 + self.message.bytes().len()];
        let context = cookie::WriteContext::from(raw_buffer);
        let context = cookie::combinator::slice(&Self::IDENTIFIER)(context)?;

        let context = cookie::bytes::be_u32(self.new_magic)(context)?;

        let context = cookie::bytes::be_u32(self.user_id)(context)?;
        let context = cookie::bytes::be_u8(self.username_len)(context)?;
        let context = cookie::combinator::string(&self.username)(context)?;

        let context = cookie::bytes::be_u16(self.message_len)(context)?;
        let context = cookie::combinator::string(&self.message)(context)?;

        Ok(context)
    }
}
