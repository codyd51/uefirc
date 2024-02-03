use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use crate::irc::Tokenizer;

#[derive(Debug, Clone)]
pub struct User(String);
#[derive(Debug, Clone)]
pub struct Channel(String);
#[derive(Debug, Clone)]
pub struct UserOrChannel(String);


#[derive(Debug, Clone)]
pub struct JoinParameters {
    user: User,
    channel: Channel,
}

impl JoinParameters {
    fn new(user: &User, channel: &Channel) -> Self {
        Self {
            user: user.clone(),
            channel: channel.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PrivateMessageParameters {
    sender: User,
    recipient: UserOrChannel,
    message: String,
}

impl PrivateMessageParameters {
    fn new(
        sender: &User,
        recipient: &UserOrChannel,
        message: &str,
    ) -> Self {
        Self {
            sender: sender.clone(),
            recipient: recipient.clone(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum IrcCommandName {
    Join,
    PrivateMessage,
    Notice,
}

impl From<&str> for IrcCommandName {
    fn from(value: &str) -> Self {
        match value {
            "JOIN" => Self::Join,
            "PRIVMSG" => Self::PrivateMessage,
            "NOTICE" => Self::Notice,
            _ => panic!("Unrecognized IRC command {value}")
        }
    }
}

#[derive(Debug, Clone)]
pub enum IrcCommand {
    Join(JoinParameters),
    PrivateMessage(PrivateMessageParameters),
}

#[derive(Debug)]
pub struct IrcMessage {
    /// May be sent by the server, but not required
    origin: Option<String>,
    command_name: IrcCommandName,
    command: IrcCommand,
}

impl IrcMessage {
    fn new(
        origin: Option<String>,
        command_name: IrcCommandName,
        command: IrcCommand,
    ) -> Self {
        Self {
            origin,
            command_name,
            command,
        }
    }
}

pub struct ResponseParser {
    buffered_data: Vec<u8>,
}

impl ResponseParser {
    pub fn new() -> Self {
        Self {
            buffered_data: vec![],
        }
    }

    pub fn ingest(&mut self, data: &[u8]) {
        self.buffered_data.extend(data)
    }

    fn read_next_line(&mut self) -> Option<String> {
        // Check whether we've got a line ready to parse
        // TODO(PT): Extract the IRC newline sequence?
        let irc_newline_seq = b"\r\n";
        let newline_pos = self.buffered_data.windows(2).position(|w| w == irc_newline_seq);
        let newline_start_idx = match newline_pos {
            // No newline ready yet
            None => return None,
            Some(p) => p
        };
        let end_of_line_idx = newline_start_idx + irc_newline_seq.len();
        let line = self.buffered_data.drain(..end_of_line_idx).collect::<Vec<u8>>();
        Some(String::from_utf8(line).expect("Failed to decode"))
    }

    pub fn parse_next_line(&mut self) -> Option<IrcMessage> {
        let line = match self.read_next_line() {
            None => return None,
            Some(line) => line,
        };

        let mut tokenizer = Tokenizer::new(&line);
        // Does this message include a prefix?
        let origin = match tokenizer.peek() == Some(':') {
            true => {
                tokenizer.match_str(":");
                Some(tokenizer.read_to(' ').expect("Failed to find space after prefix?"))
            }
            false => None,
        };

        Some(
            IrcMessage::new(
                origin,
                IrcCommandName::Join,
                IrcCommand::Join(JoinParameters::new(&User("pt".to_string()), &Channel("test".to_string())))
            )
        )
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;
    use crate::irc::{ResponseParser, Tokenizer};

    #[test]
    fn test_parse_welcome_message() {
        let mut p = ResponseParser::new();
        p.ingest(":irc.example.com 001 nick :Welcome to the IRC Network, YourNick!YourUser@your.host\r\n".as_bytes());
        let parsed_msg = p.parse_next_line().expect("Failed to parse message");

        assert_eq!(parsed_msg.origin, Some("irc.example.com".to_string()));

        assert!(p.parse_next_line().is_none());
    }
}

