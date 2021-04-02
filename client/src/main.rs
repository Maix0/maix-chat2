use common::{Packet, PacketOwned};
use std::error::Error;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::Style,
    text::{Span, Spans},
    widgets::{Paragraph, StatefulWidget, Widget, Wrap},
};

extern crate common;
extern crate crossbeam_channel;
extern crate crossterm;
extern crate tui;
// use crossbeam_channel::{Receiver, Sender};

use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen},
};

static mut CLIENT_MAGIC: u32 = 0;
static mut CLIENT_ID: u32 = 0;

#[derive(Debug, Clone)]
struct Message {
    author_id: u32,
    author_username: String,
    message: String,
}

struct MessageListWidget;

fn get_spacer(len: usize) -> &'static str {
    match len {
        0 => "          ",
        1 => "         ",
        2 => "        ",
        3 => "       ",
        4 => "      ",
        5 => "     ",
        6 => "    ",
        7 => "   ",
        8 => "  ",
        9 => " ",
        _ => "",
    }
}

impl StatefulWidget for MessageListWidget {
    type State = MessageList;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let text = state
            .inner_list
            .iter()
            .skip(state.message_scroll)
            .map(|x| {
                Spans::from(vec![
                    Span::styled(x.author_username.as_str(), x.get_username_style()),
                    Span::styled(":", Style::default().fg(tui::style::Color::White)),
                    Span::styled(
                        get_spacer(x.author_username.len()),
                        Style::default().fg(tui::style::Color::White),
                    ),
                    Span::styled(
                        x.message.as_str(),
                        Style::default().fg(tui::style::Color::White),
                    ),
                ])
            })
            .collect::<Vec<_>>();
        let paragraph = Paragraph::new(text).wrap(Wrap { trim: true });

        paragraph.render(area, buf);
    }
}

#[derive(Clone, Copy, Debug)]
pub enum SystemUserType {
    System,
    Server,
}

impl SystemUserType {
    fn get_style(&self) -> tui::style::Style {
        match self {
            SystemUserType::System => tui::style::Style::default()
                .fg(tui::style::Color::Red)
                .bg(tui::style::Color::DarkGray)
                .add_modifier(tui::style::Modifier::BOLD),
            SystemUserType::Server => tui::style::Style::default()
                .fg(tui::style::Color::Blue)
                .bg(tui::style::Color::DarkGray)
                .add_modifier(tui::style::Modifier::BOLD),
        }
    }
}

impl Message {
    pub fn is_system(&self) -> bool {
        (self.author_id & 0xF0000000) > 0
    }
    pub fn system_type(&self) -> Option<SystemUserType> {
        if self.is_system() {
            match self.author_id & 0xF0000000 {
                0xF0000000 => Some(SystemUserType::System),
                0xE0000000 => Some(SystemUserType::Server),
                _ => None,
            }
        } else {
            None
        }
    }
    pub fn get_username_style(&self) -> tui::style::Style {
        self.system_type().map(|s| s.get_style()).unwrap_or({
            let [_, r, g, b] = (self.author_id >> 4).to_be_bytes();
            tui::style::Style::default().fg(tui::style::Color::Rgb(r, b, g))
        })
    }
}

#[derive(Clone, Debug)]
struct MessageList {
    inner_list: std::collections::VecDeque<Message>,
    message_scroll: usize,
}

impl MessageList {
    const MAX_MESSAGE: usize = 100;

    pub fn new() -> Self {
        Self {
            inner_list: std::collections::VecDeque::with_capacity(Self::MAX_MESSAGE),
            message_scroll: 0,
        }
    }
    pub fn push_message(&mut self, message: Message) {
        if self.len() + 1 > Self::MAX_MESSAGE {
            self.inner_list.pop_front();
        }
        self.inner_list.push_back(message);
    }

    pub fn len(&self) -> usize {
        self.inner_list.len()
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let (_, server_ip, username) = (
        args.next(),
        {
            let username = args
                .next()
                .expect("This programe take two argument: `serverip:port` and `username`");
            username.trim()[..30.min(username.len())].to_string()
        },
        args.next()
            .expect("This programe take two argument: `serverip:port` and `username`"),
    );
    let (tx, rx) = crossbeam_channel::unbounded();
    let (sender_message, _recv_message) = crossbeam_channel::unbounded::<Message>();
    let (_sender_localmessage, recv_localmessage) = crossbeam_channel::unbounded::<Message>();

    std::thread::spawn(move || loop {
        if let CEvent::Key(key) = event::read().unwrap() {
            tx.send(key).unwrap();
        }
    });

    let server_ip2 = server_ip.clone();

    let username2 = username.clone();
    std::thread::spawn(move || {
        use common::{parser::FromBytes, serializer::IntoBytes};
        use std::{io::prelude::*, net};
        let sender_message = sender_message;
        let _recv_localmessage = recv_localmessage;

        let connection = net::TcpStream::connect(server_ip2.clone());

        if let Err(_e) = connection.as_ref() {
            sender_message
                .send(Message {
                    author_id: 0xF0_00_00_00,
                    author_username: String::from("System"),
                    message: format!("Failed to connect to server"),
                })
                .unwrap();
            return;
        }
        let mut connection = connection.unwrap();
        connection.set_nonblocking(true).unwrap();
        connection.set_nodelay(true).unwrap();
        connection
            .set_read_timeout(Some(std::time::Duration::from_secs(20)))
            .unwrap();
        let mut status = 0u8; /*
                              0 => Not connected
                              1 => Server Confirmation
                              2 => Sent Confirmation
                              3 => Done
                              */
        let mut buffer = Vec::with_capacity(256);
        let mut packets = Vec::with_capacity(10);
        loop {
            buffer.clear();
            packets.clear();

            let res = connection.read(&mut buffer);
            if let Err(_e) = res.as_ref() {
                sender_message
                    .send(Message {
                        author_id: 0xF0_00_00_00,
                        author_username: String::from("System"),
                        message: format!("Failed to connect to server"),
                    })
                    .unwrap();
                return;
            }
            let mut data = &buffer[..res.unwrap()];

            loop {
                eprint!(".");
                let res = Packet::from_bytes(data);
                if let Ok((new_data, packet)) = res {
                    data = new_data;
                    packets.push(packet.into_owned());
                    eprintln!("{:?} {:?}", &data, &packet);
                } else {
                    break;
                }
            }

            match status {
                0 => {
                    let res = connection.write(
                        &common::ClientRegistrationRequest {
                            username_len: username2.len() as u8,
                            username: username2.as_str(),
                        }
                        .unwrap_bytes()[..],
                    );
                    if let Err(_e) = res.as_ref() {
                        sender_message
                            .send(Message {
                                author_id: 0xF0_00_00_00,
                                author_username: String::from("System"),
                                message: format!("Failed to connect to server"),
                            })
                            .expect("Error when sending system message");
                        return;
                    }
                    eprintln!("fdsfq");
                    status = 1;
                }
                1 => {
                    if let Some(PacketOwned::ServerRegistrationConfirmation(packet)) =
                        packets.get(1)
                    {
                        unsafe {
                            CLIENT_ID = packet.client_id;
                            CLIENT_MAGIC = packet.magic;
                        }
                    }
                }
                2 => {}
                3 => {}

                _ => unreachable!(),
            }
        }
    });

    let mut message_list = MessageList::new();
    message_list.push_message(Message {
        author_id: 0xF0_00_00_00,
        author_username: String::from("System"),
        message: format!(
            "Connecting to `{}` with username: `{}`",
            &server_ip, username
        ),
    });

    let layout = tui::layout::Layout::default()
        .margin(0)
        .direction(tui::layout::Direction::Vertical)
        .constraints(
            [
                tui::layout::Constraint::Min(5),
                tui::layout::Constraint::Length(3),
            ]
            .as_ref(),
        );

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    enable_raw_mode()?;

    let backend = tui::backend::CrosstermBackend::new(stdout);
    let mut terminal = tui::Terminal::new(backend)?;
    let mut message_string = String::with_capacity(250);
    loop {
        match rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(key_event) => match key_event.code {
                KeyCode::Backspace | KeyCode::Delete => {
                    message_string.pop();
                }
                KeyCode::Enter => {
                    /* Placeholder */
                    if message_string.len() > 0 {
                        if unsafe { CLIENT_ID } != 0 {
                            message_list.push_message(Message {
                                author_id: unsafe { CLIENT_ID },
                                author_username: username.clone(),
                                message: message_string.clone(),
                            });
                            message_string.clear();
                            if (message_list.len() as u16) > terminal.size()?.height - 6
                                && message_list.message_scroll < MessageList::MAX_MESSAGE
                            {
                                message_list.message_scroll +=
                                    if message_list.len() == MessageList::MAX_MESSAGE {
                                        0
                                    } else {
                                        1
                                    };
                            }
                        }
                    }
                    /* Send message */
                }
                KeyCode::Down | KeyCode::PageDown => {
                    if (message_list.len() as u16) > terminal.size()?.height - 6
                        && message_list.message_scroll < MessageList::MAX_MESSAGE
                    {
                        message_list.message_scroll += 1;
                    }
                }
                KeyCode::Up | KeyCode::PageUp => {
                    message_list.message_scroll = message_list.message_scroll.saturating_sub(1)
                }
                KeyCode::Insert => {}
                KeyCode::Char(chr) => {
                    if message_string.len() <= 250 {
                        message_string.push(chr);
                    }
                }
                KeyCode::Esc => {
                    disable_raw_mode()?;
                    break;
                }
                _ => {}
            },
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                disable_raw_mode()?;
                break;
            }
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
        }

        terminal.draw(|f| {
            let rects = layout.split(f.size());
            let message_block = tui::widgets::Block::default()
                .title(format!("Messages on {}", server_ip))
                .borders(tui::widgets::Borders::ALL)
                .style(tui::style::Style::default().fg(tui::style::Color::Red));
            let message_block_inner = message_block.inner(rects[0]);

            f.render_widget(message_block, rects[0]);
            f.render_stateful_widget(MessageListWidget, message_block_inner, &mut message_list);

            let input_block = tui::widgets::Block::default()
                .title("Input")
                .borders(tui::widgets::Borders::ALL)
                .style(tui::style::Style::default().fg(tui::style::Color::Yellow));

            let input_block_inner = input_block.inner(rects[1]);
            let render_text = Paragraph::new(message_string.as_str())
                .scroll((0, {
                    let of = if (message_string.len() as u16) < input_block_inner.width {
                        0
                    } else {
                        message_string.len() as u16 - input_block_inner.width
                    };
                    f.set_cursor(
                        (message_string
                            .len()
                            .min(input_block_inner.width as usize - 1)
                            + 1) as u16,
                        input_block_inner.y,
                    );
                    of
                }))
                .style(tui::style::Style::default().fg(tui::style::Color::White))
                .block(input_block);
            f.render_widget(render_text, rects[1]);
        })?;
    }
    println!();
    Ok(())
}
