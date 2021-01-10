pub extern crate nom;
use crate::{
    ClientRegistrationRequest, ClientSendMessage, HeartBeatRequest, HeartBeatSend, Packet,
    ServerBroadcastMessage, ServerRegistrationConfirmation,
};
use nom::bytes::complete as bytes;
use nom::IResult;

pub trait FromBytes<'a>
where
    Self: 'a + Sized,
{
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Self, ParserError>;
}

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Invalid tag")]
    InvalidTag,
    #[error("Not enough data")]
    MissingData,
    #[error("Data isn't UTF8")]
    NotUTF8,
}
macro_rules! packet_def {
            ($tag:expr,$input:expr ,($($id:ident),*)) => {
                match $tag {
                    $($id::IDENTIFIER => $id::from_bytes($input).map(|(input,packet)| (input, Packet::$id(packet))),)*
                    _ => Err(nom::Err::Failure(ParserError::InvalidTag)),
                }
            };
        }
impl<'a> FromBytes<'a> for Packet<'a> {
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Packet<'a>, ParserError> {
        let (input, tag) =
            bytes::take(3usize)(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::InvalidTag)
            })?;
        let tag = {
            let mut tmp = [0; 3];
            tmp.clone_from_slice(tag);
            tmp
        };

        let (input, packet) = packet_def!(
            tag,
            input,
            (
                ClientRegistrationRequest,
                ClientSendMessage,
                HeartBeatSend,
                HeartBeatRequest,
                ServerBroadcastMessage,
                ServerRegistrationConfirmation
            )
        )?;

        Ok((input, packet))
    }
}

impl<'a> FromBytes<'a> for ClientRegistrationRequest<'a> {
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Self, ParserError> {
        let (input, username_len) =
            nom::number::complete::be_u8(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, username_bytes) =
            bytes::take(username_len)(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let username = std::str::from_utf8(username_bytes)
            .map_err(|_| nom::Err::Failure(ParserError::NotUTF8))?;

        Ok((
            input,
            ClientRegistrationRequest {
                username,
                username_len,
            },
        ))
    }
}

impl<'a> FromBytes<'a> for ClientSendMessage<'a> {
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Self, ParserError> {
        let (input, client_id) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, solved) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, message_len) =
            nom::number::complete::be_u16(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, message_bytes) =
            bytes::take(message_len)(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let message = std::str::from_utf8(message_bytes)
            .map_err(|_| nom::Err::Failure(ParserError::NotUTF8))?;

        Ok((
            input,
            ClientSendMessage {
                client_id,
                solved,
                message_len,
                message,
            },
        ))
    }
}
impl<'a> FromBytes<'a> for HeartBeatSend {
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Self, ParserError> {
        let (input, client_id) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, solved) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        Ok((input, HeartBeatSend { client_id, solved }))
    }
}

impl<'a> FromBytes<'a> for HeartBeatRequest {
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Self, ParserError> {
        let (input, new_magic) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        Ok((input, HeartBeatRequest { new_magic }))
    }
}
impl<'a> FromBytes<'a> for ServerBroadcastMessage<'a> {
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Self, ParserError> {
        let (input, new_magic) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;

        let (input, user_id) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, username_len) =
            nom::number::complete::be_u8(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, username_bytes) =
            bytes::take(username_len)(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let username = std::str::from_utf8(username_bytes)
            .map_err(|_| nom::Err::Failure(ParserError::NotUTF8))?;
        let (input, message_len) =
            nom::number::complete::be_u16(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let (input, message_bytes) =
            bytes::take(message_len)(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;
        let message = std::str::from_utf8(message_bytes)
            .map_err(|_| nom::Err::Failure(ParserError::NotUTF8))?;
        Ok((
            input,
            ServerBroadcastMessage {
                new_magic,
                user_id,
                username_len,
                username,
                message_len,
                message,
            },
        ))
    }
}

impl<'a> FromBytes<'a> for ServerRegistrationConfirmation {
    fn from_bytes(input: &'a [u8]) -> IResult<&'a [u8], Self, ParserError> {
        let (input, client_id) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;

        let (input, magic) =
            nom::number::complete::be_u32(input).map_err(|_: nom::Err<nom::error::Error<_>>| {
                nom::Err::Failure(ParserError::MissingData)
            })?;

        Ok((input, ServerRegistrationConfirmation { client_id, magic }))
    }
}
