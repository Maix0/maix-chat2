#![warn(clippy::all)]
extern crate common;
extern crate crossbeam_channel;
#[macro_use]
extern crate log;
extern crate ctrlc;
extern crate rand;
extern crate simplelog;
use std::{
    collections::{HashMap, HashSet},
    io::prelude::*,
    net,
    sync::atomic::{AtomicBool, Ordering},
};

use common::{parser::FromBytes, serializer::IntoBytes, Packet, PacketOwned};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum ConnectionStatus {
    WaitingForClientVerification = 0,
    SentServerConfirmation = 1,
    HandShakeDone = 2,
}

type ClientID = u32;
struct Client {
    pub(crate) con: net::TcpStream,
    pub(crate) id: ClientID,
    pub(crate) connection_status: ConnectionStatus,
    pub(crate) heartbeat_skipped: u8,
    pub(crate) lastheart_beat: std::time::Instant,
    pub(crate) magic: u32,
    pub(crate) username: String,
}

impl Client {
    fn send_hearbeat(&mut self) -> Result<(), std::io::Error> {
        let packet = common::HeartBeatRequest {};
        let bytes = packet.unwrap_bytes();
        self.con.write(&bytes)?;
        Ok(())
    }

    fn send_registration_confirmation(&mut self) -> Result<(), std::io::Error> {
        let packet = common::ServerRegistrationConfirmation {
            client_id: self.id,
            magic: self.magic,
        };
        let bytes = packet.unwrap_bytes();
        self.con.write(&bytes)?;
        Ok(())
    }
}

// the message type for the message broadcast queue
type Message = Vec<u8>;

// the maximum size of a packet in bytes;
// Currently the largest packet is commom::ServerBroadcastMessage
const PACKET_MAX_SIZE: usize = 3 + 4 + 4 + 1 + u8::MAX as usize + 2 + u16::MAX as usize;

const MAX_HB_SKIP: u8 = 5;
const HB_SKIP_REST: std::time::Duration = std::time::Duration::from_secs(30);
const HB_REQUEST_TIME: std::time::Duration = std::time::Duration::from_secs(2);
static STOPPING: AtomicBool = AtomicBool::new(false);

fn main() {
    println!();
    simplelog::TermLogger::init(
        simplelog::LevelFilter::max(),
        Default::default(),
        simplelog::TerminalMode::Mixed,
    )
    .unwrap();

    let _ = ctrlc::set_handler(|| {
        info!("Stopping the server!");
        STOPPING.store(true, Ordering::SeqCst);
    })
    .map_err(|e| error!("Error when setting shutdown handler: {}", e));

    let (_thread_handle, recv_tcp) = {
        let (h, rx) = generate_connection_handler("127.0.0.1:8888");
        (std::thread::spawn(h), rx)
    };

    // The last "skipped hb" clear
    let mut last_clear = std::time::Instant::now();

    // List of all clients with default capacity of 100;
    let mut clients: HashMap<ClientID, Client> = HashMap::with_capacity(10);

    // List of all packet that will be handled each loop
    let mut packets: Vec<(ClientID, PacketOwned)> = Vec::with_capacity(10);

    // List of all client that will be drop
    let mut to_drop: HashSet<ClientID> = HashSet::with_capacity(10);

    // A buffer to read packets;
    let mut packet_buffer: Vec<u8> = Vec::with_capacity(PACKET_MAX_SIZE);

    // The list of client in need of an heartbeat
    let mut need_hearbeat: Vec<ClientID> = Vec::with_capacity(10);

    // List of all message to broadcast
    let mut message_to_broadcast: Vec<Message> = Vec::with_capacity(10);

    'mainloop: loop {
        let need_clear_hb_skip = last_clear + HB_SKIP_REST > std::time::Instant::now();
        // Clearing the per loop list;
        packets.clear();
        to_drop.clear();
        need_hearbeat.clear();
        message_to_broadcast.clear();

        // Check for new client
        while let Ok(new_client) = recv_tcp.try_recv() {
            // Generate a id for the new client
            let mut new_id: ClientID = generate_client_id();
            // if the id already exist, generate a new one
            while clients.get(&new_id).is_some() {
                new_id = generate_client_id();
            }
            let magic = generate_client_magic();
            // Add new client to the clients hashmap
            clients.insert(
                new_id,
                Client {
                    con: new_client,
                    id: new_id,
                    connection_status: ConnectionStatus::WaitingForClientVerification,
                    heartbeat_skipped: 0,
                    lastheart_beat: std::time::Instant::now(),
                    magic,
                    username: String::new(),
                },
            );
        }
        // get a list of all the packets that the clients send
        for (&client_id, client) in clients.iter_mut() {
            if need_clear_hb_skip {
                client.heartbeat_skipped = 0;
            }
            packet_buffer.clear();
            client
                .con
                .read_to_end(&mut packet_buffer)
                .map_err(|err| {
                    error!(
                        "Error when reading data from client `{}`: {}",
                        client_id, err
                    );
                    to_drop.insert(client_id);
                })
                .unwrap_or(0);
            let mut bytes = packet_buffer.as_slice();
            while let Ok((new_bytes, packet)) = Packet::from_bytes(bytes) {
                bytes = new_bytes;
                packets.push((client_id, packet.into_owned()));
            }
        }

        // process the packet of the clients
        for (client_id, packet) in packets.drain(..) {
            let client = clients.get_mut(&client_id);
            if client.is_none() {
                to_drop.insert(client_id);
                continue;
            }
            let client = client.unwrap();
            match packet {
                PacketOwned::ClientRegistrationRequest(packet) => {
                    // If the client is already registered, wrong packet => dropped
                    if client.connection_status != ConnectionStatus::WaitingForClientVerification {
                        info!("Client `{}` sent wrong packet", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }
                    // Set the values
                    client.username = packet.username;
                    client.connection_status = ConnectionStatus::SentServerConfirmation;

                    // Sending him the next registration packet
                    if let Err(e) = client.send_registration_confirmation() {
                        error!("Error when sending packet to client `{}`: {}", client_id, e);
                        to_drop.insert(client_id);
                        continue;
                    }
                }
                PacketOwned::ClientRegistrationEnd(packet) => {
                    // If the client is already registered, wrong packet => dropped
                    if client.connection_status != ConnectionStatus::SentServerConfirmation {
                        info!("Client `{}` sent wrong packet", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }
                    // Checking if the client sent the correct id
                    if client.id != packet.client_id {
                        debug!("Client `{}` sent wrong id", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }
                    // Checking if the client sent the correct magic
                    if client.magic != packet.magic {
                        debug!("Client `{}` sent wrong magic", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }

                    client.connection_status = ConnectionStatus::HandShakeDone;
                }
                PacketOwned::ClientSendMessage(packet) => {
                    // If the client is already registered, wrong packet => dropped
                    if client.connection_status != ConnectionStatus::HandShakeDone {
                        info!("Client `{}` sent wrong packet", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }
                    // Checking if the client sent the correct id
                    if client.id != packet.client_id {
                        debug!("Client `{}` sent wrong id", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }
                    // Checking if the client sent the correct magic
                    if client.magic != packet.magic {
                        debug!("Client `{}` sent wrong magic", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }
                    // Construct the "Message" to broadcast to other clients
                    let message_packet = common::ServerBroadcastMessage {
                        user_id: client.id,
                        username: client.username.as_str(),
                        username_len: client.username.len() as u8,
                        message: packet.message.as_str(),
                        message_len: packet.message.len() as u16,
                    };

                    message_to_broadcast.push(message_packet.unwrap_bytes());
                }
                PacketOwned::HeartBeatSend(_) => {
                    // If the client is already registered, wrong packet => dropped
                    if client.connection_status != ConnectionStatus::HandShakeDone {
                        info!("Client `{}` sent wrong packet", client_id);
                        to_drop.insert(client_id);
                        continue;
                    }

                    client.lastheart_beat = std::time::Instant::now();
                    trace!("Got HeartBeat from client `{}`", client_id);
                }

                // Only server sending these packets => dropping client
                PacketOwned::ServerRegistrationConfirmation(_)
                | PacketOwned::ServerBroadcastMessage(_)
                | PacketOwned::HeartBeatRequest(_) => {
                    debug!("`{}` sent a server-only packet, dropping him", client_id);
                    to_drop.insert(client_id);
                }
                _ => error!("Packet: {:?} isn\'t supported", packet.get_identifier()),
            }
        }

        // Looping over every client and message to broadcast them
        for client in clients.values_mut() {
            for message in &message_to_broadcast {
                if let Err(e) = client.con.write(message) {
                    error!("Error when sending packet to client `{}`: {}", client.id, e);
                    to_drop.insert(client.id);
                }
            }

            if client.lastheart_beat + HB_REQUEST_TIME > std::time::Instant::now() {
                client.heartbeat_skipped += 1;
                if client.heartbeat_skipped >= MAX_HB_SKIP {
                    debug!("Client `{}` not responding to HeatBeat", client.id);
                    to_drop.insert(client.id);
                }
                if let Err(e) = client.send_hearbeat() {
                    error!("Error when sending packet to client `{}`: {}", client.id, e);
                    to_drop.insert(client.id);
                }
            }
        }

        for client_id in &need_hearbeat {
            let client = clients.get_mut(client_id);
            if client.is_none() {
                continue;
            }
            let client = client.unwrap();
            if let Err(e) = client.send_hearbeat() {
                error!("Error when sending packet to client `{}`: {}", client_id, e);
                to_drop.insert(*client_id);
            }
        }

        // Dropping every client in the drop list
        for client_id in &to_drop {
            clients.remove(client_id);
        }

        if need_clear_hb_skip {
            last_clear = std::time::Instant::now();
        }

        if STOPPING.load(Ordering::SeqCst) {
            break 'mainloop;
        }
    }
}

fn generate_client_id() -> ClientID {
    use rand::prelude::*;
    rand::thread_rng().gen_range(0x00000000..=0xFF000000) << 1
}
fn generate_client_magic() -> u32 {
    use rand::prelude::*;
    rand::thread_rng().gen()
}

fn generate_connection_handler(
    ip: impl net::ToSocketAddrs,
) -> (
    impl FnOnce() -> (),
    crossbeam_channel::Receiver<net::TcpStream>,
) {
    let (sx, rx) = crossbeam_channel::unbounded();

    (
        move || {
            let listener = net::TcpListener::bind(ip);
            if let Err(e) = listener.as_ref() {
                error!("Error when binding to given IP: {}", e);
                return;
            }
            let listener = listener.unwrap();
            match listener.local_addr() {
                Ok(ip) => info!("Binded on {}", ip),
                Err(e) => warn!("Unable to get local binding: {}", e),
            }
            for con in listener.incoming() {
                if STOPPING.load(Ordering::SeqCst) {
                    break;
                }
                if let Err(e) = con.as_ref() {
                    error!("Error when client connected: {}", e);
                    continue;
                }
                let con = con.unwrap();
                if let Ok(addr) = con.peer_addr() {
                    debug!("Accepted new connection: [{}]", addr);
                } else {
                    debug!("Accepted new connection: [NO ADDR]",);
                }
                match sx.send(con) {
                    Ok(_) => {}
                    Err(e) => error!("Error when passing connection: {}", e),
                };
            }
        },
        rx,
    )
}
