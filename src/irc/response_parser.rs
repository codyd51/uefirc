use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::{format, vec};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use crate::irc::Tokenizer;

const IRC_LINE_DELIMITER: &'static str = "\r\n";

#[derive(Debug, Clone, PartialEq)]
pub struct Nickname(String);

impl Display for Nickname {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(&format!("{}", self.0))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct User(String);
#[derive(Debug, Clone, PartialEq)]
pub struct Channel(String);
#[derive(Debug, Clone, PartialEq)]
pub struct UserOrChannel(String);


#[derive(Debug, Clone, PartialEq)]
pub struct JoinParameters {
    channel: Channel,
}

impl JoinParameters {
    fn new(channel: &Channel) -> Self {
        Self {
            channel: channel.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyWelcomeParams {
    pub nick: Nickname,
    pub message: String,
}

impl ReplyWelcomeParams {
    fn new(nick: &Nickname, message: &str) -> Self {
        Self {
            nick: nick.clone(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyYourHostParams {
    pub nick: Nickname,
    pub message: String,
}

impl ReplyYourHostParams {
    fn new(nick: &Nickname, message: &str) -> Self {
        Self {
            nick: nick.clone(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyCreatedParams {
    pub nick: Nickname,
    pub message: String,
}

impl ReplyCreatedParams {
    fn new(nick: &Nickname, message: &str) -> Self {
        Self {
            nick: nick.clone(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyMyInfoParams {
    pub nick: Nickname,
    pub server_name: String,
    pub version: String,
    pub available_user_modes: String,
    pub available_channel_modes: String,
    pub channel_modes_with_params: Option<String>,
}

impl ReplyMyInfoParams {
    fn new(
        nick: &Nickname,
        server_name: &str,
        version: &str,
        available_user_modes: &str,
        available_channel_modes: &str,
        channels_modes_with_params: Option<&str>,
    ) -> Self {
        Self {
            nick: nick.clone(),
            server_name: server_name.to_string(),
            version: version.to_string(),
            available_user_modes: available_user_modes.to_string(),
            available_channel_modes: available_channel_modes.to_string(),
            channel_modes_with_params: channels_modes_with_params.map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyISupportParams {
    pub nick: Nickname,
    // PT: I'm not bothering to parse these any deeper for now
    pub entries: Vec<String>,
}

impl ReplyISupportParams {
    fn new(nickname: &Nickname, entries: &[String]) -> Self {
        Self {
            nick: nickname.clone(),
            entries: entries.to_vec(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
#[derive(PartialEq)]
pub enum IrcCommandName {
    ReplyWelcome,
    ReplyYourHost,
    ReplyCreated,
    ReplyMyInfo,
    ReplyISupport,
    Join,
    PrivateMessage,
    Notice,
}

impl From<&str> for IrcCommandName {
    fn from(value: &str) -> Self {
        match value {
            "001" => Self::ReplyWelcome,
            "002" => Self::ReplyYourHost,
            "003" => Self::ReplyCreated,
            "004" => Self::ReplyMyInfo,
            "005" => Self::ReplyISupport,
            "JOIN" => Self::Join,
            "PRIVMSG" => Self::PrivateMessage,
            "NOTICE" => Self::Notice,
            _ => panic!("Unrecognized IRC command {value}")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrcCommand {
    ReplyWelcome(ReplyWelcomeParams),
    ReplyYourHost(ReplyYourHostParams),
    ReplyCreated(ReplyCreatedParams),
    ReplyMyInfo(ReplyMyInfoParams),
    ReplyISupport(ReplyISupportParams),
    Join(JoinParameters),
    PrivateMessage(PrivateMessageParameters),
}

#[derive(Debug)]
pub struct IrcMessage {
    /// May be sent by the server, but not required
    pub origin: Option<String>,
    pub command_name: IrcCommandName,
    pub command: IrcCommand,
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
        let irc_newline_seq = IRC_LINE_DELIMITER.as_bytes();
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

    fn parse_nickname(tokenizer: &mut Tokenizer) -> Nickname {
        Nickname(tokenizer.read_to(' ').expect("Failed to read nick"))
    }

    fn parse_trailing_message(tokenizer: &mut Tokenizer) -> String {
        tokenizer.match_str(":");
        tokenizer.read_to_str(IRC_LINE_DELIMITER).expect("Failed to read a message")
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

        let raw_command_name = tokenizer.read_to(' ').expect("Failed to read a command");
        let command_name = IrcCommandName::from(&raw_command_name as &str);

        let command = match command_name {
            IrcCommandName::ReplyWelcome => {
                let nick = Self::parse_nickname(&mut tokenizer);
                let message = Self::parse_trailing_message(&mut tokenizer);
                IrcCommand::ReplyWelcome(ReplyWelcomeParams::new(&nick, &message))
            }
            IrcCommandName::ReplyYourHost => {
                let nick = Self::parse_nickname(&mut tokenizer);
                let message = Self::parse_trailing_message(&mut tokenizer);
                IrcCommand::ReplyYourHost(ReplyYourHostParams::new(&nick, &message))
            }
            IrcCommandName::ReplyCreated => {
                let nick = Self::parse_nickname(&mut tokenizer);
                let message = Self::parse_trailing_message(&mut tokenizer);
                IrcCommand::ReplyCreated(ReplyCreatedParams::new(&nick, &message))
            }
            IrcCommandName::ReplyMyInfo => {
                let nick = Self::parse_nickname(&mut tokenizer);
                let server = tokenizer.read_to(' ').expect("Failed to read server");
                let version = tokenizer.read_to(' ').expect("Failed to read version");
                let available_umodes = tokenizer.read_to(' ').expect("Failed to read available user modes");
                let available_cmodes = tokenizer.read_to_any(&[" ", IRC_LINE_DELIMITER]).expect("Failed to read available channel modes");
                let cmodes_with_params = tokenizer.read_to_any(&[" ", IRC_LINE_DELIMITER]);
                IrcCommand::ReplyMyInfo(
                    ReplyMyInfoParams::new(
                        &nick,
                        &server,
                        &version,
                        &available_umodes,
                        &available_cmodes,
                        cmodes_with_params.as_ref().map(String::as_str),
                    )
                )
            }
            IrcCommandName::ReplyISupport => {
                let nick = Self::parse_nickname(&mut tokenizer);
                let mut entries = vec![];
                loop {
                    let capability = tokenizer.read_to(' ').expect("Failed to read capability");
                    entries.push(capability);
                    match tokenizer.peek() {
                        None => break,
                        Some(ch) => {
                            if ch == ':' {
                                tokenizer.match_str(":are supported by this server");
                                break;
                            }
                        }
                    }
                }
                IrcCommand::ReplyISupport(ReplyISupportParams::new(&nick, &entries))
            }
            IrcCommandName::Join => {
                let channel = tokenizer.read_to_str(IRC_LINE_DELIMITER).expect("Failed to read a channel name");
                if channel.contains(" ") {
                    // Only clients can specify multiple channels
                    panic!("Multiple channels mentioned, servers should not send multiple channels?")
                }
                IrcCommand::Join(JoinParameters::new(&Channel(channel)))
            }
            IrcCommandName::Notice => {
                todo!()
            },
            IrcCommandName::PrivateMessage => todo!(),
        };

        Some(
            IrcMessage::new(
                origin,
                command_name,
                command,
            )
        )
    }
}

#[cfg(test)]
mod test {
    use alloc::string::ToString;
    use crate::irc::{ResponseParser};
    use crate::irc::response_parser::{Channel, IrcCommand, IrcCommandName, IrcMessage, JoinParameters, Nickname, ReplyCreatedParams, ReplyISupportParams, ReplyMyInfoParams, ReplyWelcomeParams, ReplyYourHostParams};

    fn parse_line(line: &str) -> IrcMessage {
        let mut p = ResponseParser::new();
        p.ingest(line.as_bytes());
        let parsed_msg = p.parse_next_line().expect("Failed to parse message");
        assert!(p.parse_next_line().is_none());
        parsed_msg
    }

    #[test]
    fn test_parse_multiple_lines() {
        let mut p = ResponseParser::new();
        p.ingest("JOIN #chan1\r\nJOIN #chan2\r\n".as_bytes());
        let msg1 = p.parse_next_line().unwrap();
        assert_eq!(msg1.origin, None);
        assert_eq!(msg1.command_name, IrcCommandName::Join);
        assert_eq!(msg1.command, IrcCommand::Join(JoinParameters::new(&Channel("#chan1".to_string()))));
        let msg2 = p.parse_next_line().unwrap();
        assert_eq!(msg2.origin, None);
        assert_eq!(msg2.command_name, IrcCommandName::Join);
        assert_eq!(msg2.command, IrcCommand::Join(JoinParameters::new(&Channel("#chan2".to_string()))));

        assert!(p.parse_next_line().is_none());
    }

    #[test]
    fn test_parse_welcome() {
        let msg = parse_line(":irc.example.com 001 phill :Welcome to the IRC Network, phill!s@localhost\r\n");
        assert_eq!(msg.origin, Some("irc.example.com".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyWelcome);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyWelcome(
                ReplyWelcomeParams::new(
                    &Nickname("phill".to_string()),
                    "Welcome to the IRC Network, phill!s@localhost",
                )
            )
        );
    }

    #[test]
    fn test_parse_your_host() {
        let msg = parse_line(":irc.example.com 002 phill :Your host is irc.example.com, running version fake\r\n");
        assert_eq!(msg.origin, Some("irc.example.com".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyYourHost);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyYourHost(
                ReplyYourHostParams::new(
                    &Nickname("phill".to_string()),
                    "Your host is irc.example.com, running version fake",
                )
            )
        );
    }

    #[test]
    fn test_parse_created() {
        let msg = parse_line(":irc.example.com 003 phill :This server was created on caffeine\r\n");
        assert_eq!(msg.origin, Some("irc.example.com".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyCreated);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyCreated(
                ReplyCreatedParams::new(
                    &Nickname("phill".to_string()),
                    "This server was created on caffeine",
                )
            )
        );
    }

    #[test]
    fn test_parse_my_info() {
        // One message that does specify the channels with parameters
        let msg = parse_line(":copper.libera.chat 004 phillipt copper.libera.chat solanum-1.0-dev DGIMQRSZaghilopsuwz CFILMPQRSTbcefgijklmnopqrstuvz bkloveqjfI\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyMyInfo);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyMyInfo(
                ReplyMyInfoParams::new(
                    &Nickname("phillipt".to_string()),
                    &"copper.libera.chat",
                    &"solanum-1.0-dev",
                    &"DGIMQRSZaghilopsuwz",
                    &"CFILMPQRSTbcefgijklmnopqrstuvz",
                    Some(&"bkloveqjfI"),
                )
            )
        );

        // And a message that doesn't specify the channels with parameters
        let msg = parse_line(":copper.libera.chat 004 phillipt copper.libera.chat solanum-1.0-dev DGIMQRSZaghilopsuwz CFILMPQRSTbcefgijklmnopqrstuvz\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyMyInfo);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyMyInfo(
                ReplyMyInfoParams::new(
                    &Nickname("phillipt".to_string()),
                    &"copper.libera.chat",
                    &"solanum-1.0-dev",
                    &"DGIMQRSZaghilopsuwz",
                    &"CFILMPQRSTbcefgijklmnopqrstuvz",
                    None,
                )
            )
        );
    }

    #[test]
    fn test_parse_i_support() {
        let msg = parse_line(":copper.libera.chat 005 phillipt ACCOUNTEXTBAN=a ETRACE FNC WHOX KNOCK CALLERID=g MONITOR=100 SAFELIST ELIST=CMNTU CHANTYPES=# EXCEPTS INVEX :are supported by this server\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyISupport);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyISupport(
                ReplyISupportParams::new(
                    &Nickname("phillipt".to_string()),
                    &[
                        "ACCOUNTEXTBAN=a".to_string(),
                        "ETRACE".to_string(),
                        "FNC".to_string(),
                        "WHOX".to_string(),
                        "KNOCK".to_string(),
                        "CALLERID=g".to_string(),
                        "MONITOR=100".to_string(),
                        "SAFELIST".to_string(),
                        "ELIST=CMNTU".to_string(),
                        "CHANTYPES=#".to_string(),
                        "EXCEPTS".to_string(),
                        "INVEX".to_string(),
                    ],
                ),
            )
        )
    }
}
