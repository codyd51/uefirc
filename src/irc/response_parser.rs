use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::{format, vec};
use alloc::vec::Vec;
use core::fmt::{Display, Formatter};
use crate::irc::Tokenizer;

const IRC_LINE_DELIMITER: &'static str = "\r\n";

#[derive(Debug, Clone, PartialEq)]
pub struct Nickname(String);

impl Nickname {
    fn new(nick: &str) -> Self {
        Self(nick.to_string())
    }
}

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
pub struct ReplyWithNickAndMessageParams {
    pub nick: Nickname,
    pub message: String,
}

impl ReplyWithNickAndMessageParams {
    fn new(nick: &Nickname, message: &str) -> Self {
        Self {
            nick: nick.clone(),
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DescriptorAndReasonParams {
    pub descriptor: String,
    pub reason: String,
}

impl DescriptorAndReasonParams {
    fn new(descriptor: &str, reason: &str) -> Self {
        Self {
            descriptor: descriptor.to_string(),
            reason: reason.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorUnknownCommandParams {
    pub nick: Nickname,
    pub command: String,
    pub message: String,
}

impl ErrorUnknownCommandParams {
    fn new(nick: Nickname, command: &str, message: &str) -> Self {
        Self {
            nick,
            command: command.to_string(),
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
pub struct ReplyListOperatorUsersParams {
    pub nickname: Nickname,
    pub operator_count: usize,
    pub message: String,
}

impl ReplyListOperatorUsersParams {
    fn new(nickname: &Nickname, operator_count: usize, message: &str) -> Self {
        Self {
            nickname: nickname.clone(),
            operator_count,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyListUnknownUsersParams {
    pub nickname: Nickname,
    pub unknown_user_count: usize,
    pub message: String,
}

impl ReplyListUnknownUsersParams {
    fn new(nickname: &Nickname, unknown_user_count: usize, message: &str) -> Self {
        Self {
            nickname: nickname.clone(),
            unknown_user_count,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyListChannelsParams {
    pub nickname: Nickname,
    pub channel_count: usize,
    pub message: String,
}

impl ReplyListChannelsParams {
    fn new(nickname: &Nickname, channel_count: usize, message: &str) -> Self {
        Self {
            nickname: nickname.clone(),
            channel_count,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyLocalUsersParams {
    pub nick: Nickname,
    pub current_count: usize,
    pub max_count: usize,
    pub message: String,
}

impl ReplyLocalUsersParams {
    fn new(
        nickname: &Nickname,
        current_count: usize,
        max_count: usize,
        message: &str,
    ) -> Self {
        Self {
            nick: nickname.clone(),
            current_count,
            max_count,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplyGlobalUsersParams {
    pub nick: Nickname,
    pub current_count: usize,
    pub max_count: usize,
    pub message: String,
}

impl ReplyGlobalUsersParams {
    fn new(
        nickname: &Nickname,
        current_count: usize,
        max_count: usize,
        message: &str,
    ) -> Self {
        Self {
            nick: nickname.clone(),
            current_count,
            max_count,
            message: message.to_string(),
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

#[derive(Debug, Clone, PartialEq)]
pub struct ModeParams {
    pub nick: Nickname,
    // PT: Not bothering to parse this deeper for now
    pub mode: String,
}

impl ModeParams {
    fn new(
        nick: &Nickname,
        mode: &str,
    ) -> Self {
        Self {
            nick: nick.clone(),
            mode: mode.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PingParams {
    server: String,
}

impl PingParams {
    fn new(server: &str) -> Self {
        Self {
            server: server.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuitParams {
    reason: String,
}

impl QuitParams {
    fn new(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrorParams {
    reason: String,
}

impl ErrorParams {
    fn new(reason: &str) -> Self {
        Self {
            reason: reason.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NoticeParams {
    pub target: String,
    pub message: String,
}

impl NoticeParams {
    fn new(target: &str, message: &str) -> Self {
        Self {
            target: target.to_string(),
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
    ReplyListClientUsers,
    ReplyListOperatorUsers,
    ReplyListUnknownUsers,
    ReplyListChannels,
    ReplyListUserMe,
    ReplyLocalUsers,
    ReplyGlobalUsers,
    ReplyConnectionStats,
    ReplyMessageOfTheDayStart,
    ReplyMessageOfTheDayLine,
    ReplyMessageOfTheDayEnd,
    ErrorNoSuchNick,
    ErrorUnknownCommand,
    Mode,
    Ping,
    Quit,
    Error,
    Notice,
    Join,
    PrivateMessage,
}

impl From<&str> for IrcCommandName {
    fn from(value: &str) -> Self {
        match value {
            "001" => Self::ReplyWelcome,
            "002" => Self::ReplyYourHost,
            "003" => Self::ReplyCreated,
            "004" => Self::ReplyMyInfo,
            "005" => Self::ReplyISupport,
            "251" => Self::ReplyListClientUsers,
            "252" => Self::ReplyListOperatorUsers,
            "253" => Self::ReplyListUnknownUsers,
            "254" => Self::ReplyListChannels,
            "255" => Self::ReplyListUserMe,
            "265" => Self::ReplyLocalUsers,
            "266" => Self::ReplyGlobalUsers,
            "250" => Self::ReplyConnectionStats,
            "375" => Self::ReplyMessageOfTheDayStart,
            "372" => Self::ReplyMessageOfTheDayLine,
            "376" => Self::ReplyMessageOfTheDayEnd,
            "401" => Self::ErrorNoSuchNick,
            "421" => Self::ErrorUnknownCommand,
            "MODE" => Self::Mode,
            "PING" => Self::Ping,
            "QUIT" => Self::Quit,
            "ERROR" => Self::Error,
            "NOTICE" => Self::Notice,
            "JOIN" => Self::Join,
            "PRIVMSG" => Self::PrivateMessage,
            _ => panic!("Unrecognized IRC command {value}")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IrcCommand {
    ReplyWelcome(ReplyWithNickAndMessageParams),
    ReplyYourHost(ReplyWithNickAndMessageParams),
    ReplyCreated(ReplyWithNickAndMessageParams),
    ReplyMyInfo(ReplyMyInfoParams),
    ReplyISupport(ReplyISupportParams),
    ReplyListClientUsers(ReplyWithNickAndMessageParams),
    ReplyListOperatorUsers(ReplyListOperatorUsersParams),
    ReplyListUnknownUsers(ReplyListUnknownUsersParams),
    ReplyListChannels(ReplyListChannelsParams),
    ReplyListUserMe(ReplyWithNickAndMessageParams),
    ReplyLocalUsers(ReplyLocalUsersParams),
    ReplyGlobalUsers(ReplyGlobalUsersParams),
    ReplyConnectionStats(ReplyWithNickAndMessageParams),
    ReplyMessageOfTheDayStart(ReplyWithNickAndMessageParams),
    ReplyMessageOfTheDayLine(ReplyWithNickAndMessageParams),
    ReplyMessageOfTheDayEnd(ReplyWithNickAndMessageParams),
    ErrorNoSuchNick(DescriptorAndReasonParams),
    ErrorUnknownCommand(ErrorUnknownCommandParams),
    Mode(ModeParams),
    Ping(PingParams),
    Quit(QuitParams),
    Error(ErrorParams),
    Notice(NoticeParams),
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

    fn parse_word(tokenizer: &mut Tokenizer) -> String {
        tokenizer.read_to(' ').expect("Failed to read word")
    }

    fn parse_trailing_message(tokenizer: &mut Tokenizer) -> String {
        tokenizer.match_str(":");
        tokenizer.read_to_str(IRC_LINE_DELIMITER).expect("Failed to read a message")
    }

    fn parse_usize(tokenizer: &mut Tokenizer) -> usize {
        let val_str = tokenizer.read_to(' ').expect("Failed to read a word");
        usize::from_str_radix(&val_str, 10).expect("Failed to parse a usize")
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
                IrcCommand::ReplyWelcome(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    ),
                )
            }
            IrcCommandName::ReplyYourHost => {
                IrcCommand::ReplyYourHost(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    ),
                )
            }
            IrcCommandName::ReplyCreated => {
                IrcCommand::ReplyCreated(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    ),
                )
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
            IrcCommandName::ReplyListClientUsers => {
                IrcCommand::ReplyListClientUsers(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    ),
                )
            }
            IrcCommandName::ReplyListOperatorUsers => {
                IrcCommand::ReplyListOperatorUsers(
                    ReplyListOperatorUsersParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        Self::parse_usize(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyListUnknownUsers => {
                IrcCommand::ReplyListUnknownUsers(
                    ReplyListUnknownUsersParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        Self::parse_usize(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyListChannels => {
                IrcCommand::ReplyListChannels(
                    ReplyListChannelsParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        Self::parse_usize(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyListUserMe => {
                IrcCommand::ReplyListUserMe(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyLocalUsers => {
                IrcCommand::ReplyLocalUsers(
                    ReplyLocalUsersParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        Self::parse_usize(&mut tokenizer),
                        Self::parse_usize(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyGlobalUsers => {
                IrcCommand::ReplyGlobalUsers(
                    ReplyGlobalUsersParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        Self::parse_usize(&mut tokenizer),
                        Self::parse_usize(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyConnectionStats => {
                IrcCommand::ReplyConnectionStats(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyMessageOfTheDayStart => {
                IrcCommand::ReplyMessageOfTheDayStart(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyMessageOfTheDayLine => {
                IrcCommand::ReplyMessageOfTheDayLine(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ReplyMessageOfTheDayEnd => {
                IrcCommand::ReplyMessageOfTheDayEnd(
                    ReplyWithNickAndMessageParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::ErrorNoSuchNick => {
                IrcCommand::ErrorNoSuchNick(
                    DescriptorAndReasonParams::new(
                        &tokenizer.read_to_str(" :").expect("Failed to read descriptor"),
                        &tokenizer.read_to_str(IRC_LINE_DELIMITER).expect("Failed to read a message"),
                    )
                )
            }
            IrcCommandName::ErrorUnknownCommand => {
                IrcCommand::ErrorUnknownCommand(
                    ErrorUnknownCommandParams::new(
                        Self::parse_nickname(&mut tokenizer),
                        &Self::parse_word(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::Mode => {
                IrcCommand::Mode(
                    ModeParams::new(
                        &Self::parse_nickname(&mut tokenizer),
                        &Self::parse_trailing_message(&mut tokenizer),
                    )
                )
            }
            IrcCommandName::Ping => {
                IrcCommand::Ping(
                    PingParams::new(&Self::parse_trailing_message(&mut tokenizer)),
                )
            }
            IrcCommandName::Quit => {
                IrcCommand::Quit(
                    QuitParams::new(&Self::parse_trailing_message(&mut tokenizer)),
                )
            }
            IrcCommandName::Error => {
                IrcCommand::Error(
                    ErrorParams::new(&Self::parse_trailing_message(&mut tokenizer)),
                )
            }
            IrcCommandName::Notice => {
                IrcCommand::Notice(
                    NoticeParams::new(
                        &tokenizer.read_to(' ').expect("Failed to read target"),
                        &Self::parse_trailing_message(&mut tokenizer),
                    ),
                )
            },
            IrcCommandName::Join => {
                let channel = tokenizer.read_to_str(IRC_LINE_DELIMITER).expect("Failed to read a channel name");
                if channel.contains(" ") {
                    // Only clients can specify multiple channels
                    panic!("Multiple channels mentioned, servers should not send multiple channels?")
                }
                IrcCommand::Join(JoinParameters::new(&Channel(channel)))
            }
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
    use crate::irc::{ReplyGlobalUsersParams, ReplyListChannelsParams, ReplyWithNickAndMessageParams, ReplyListOperatorUsersParams, ReplyListUnknownUsersParams, ReplyLocalUsersParams, ResponseParser, ModeParams, PingParams, QuitParams, ErrorParams, DescriptorAndReasonParams, ErrorUnknownCommandParams};
    use crate::irc::response_parser::{Channel, IrcCommand, IrcCommandName, IrcMessage, JoinParameters, Nickname, ReplyISupportParams, ReplyMyInfoParams};

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
                ReplyWithNickAndMessageParams::new(
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
                ReplyWithNickAndMessageParams::new(
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
                ReplyWithNickAndMessageParams::new(
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

    #[test]
    fn test_parse_list_client_users() {
        let msg = parse_line(":copper.libera.chat 251 phillipt :There are 68 users and 33291 invisible on 28 servers\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyListClientUsers);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyListClientUsers(ReplyWithNickAndMessageParams::new(
                &Nickname::new("phillipt"),
                "There are 68 users and 33291 invisible on 28 servers",
            ))
        )
    }

    #[test]
    fn test_parse_list_operator_users() {
        let msg = parse_line(":copper.libera.chat 252 phillipt 40 :IRC Operators online\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyListOperatorUsers);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyListOperatorUsers(ReplyListOperatorUsersParams::new(
                &Nickname::new("phillipt"),
                40,
                "IRC Operators online",
            ))
        )
    }

    #[test]
    fn test_parse_list_unknown_users() {
        let msg = parse_line(":copper.libera.chat 253 phillipt 90 :unknown connection(s)\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyListUnknownUsers);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyListUnknownUsers(ReplyListUnknownUsersParams::new(
                &Nickname::new("phillipt"),
                90,
                "unknown connection(s)",
            ))
        )
    }

    #[test]
    fn test_parse_list_channels() {
        let msg = parse_line(":copper.libera.chat 254 phillipt 22650 :channels formed\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyListChannels);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyListChannels(ReplyListChannelsParams::new(
                &Nickname::new("phillipt"),
                22650,
                "channels formed",
            ))
        )
    }

    #[test]
    fn test_parse_list_user_me() {
        let msg = parse_line(":copper.libera.chat 255 phillipt :I have 2192 clients and 1 servers\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyListUserMe);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyListUserMe(ReplyWithNickAndMessageParams::new(
                &Nickname::new("phillipt"),
                "I have 2192 clients and 1 servers",
            ))
        )
    }

    #[test]
    fn test_parse_local_users() {
        let msg = parse_line(":copper.libera.chat 265 phillipt 2192 2366 :Current local users 2192, max 2366\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyLocalUsers);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyLocalUsers(ReplyLocalUsersParams::new(
                &Nickname::new("phillipt"),
                2192,
                2366,
                "Current local users 2192, max 2366",
            ))
        )
    }

    #[test]
    fn test_parse_global_users() {
        let msg = parse_line(":copper.libera.chat 266 phillipt 33359 36895 :Current global users 33359, max 36895\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyGlobalUsers);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyGlobalUsers(ReplyGlobalUsersParams::new(
                &Nickname::new("phillipt"),
                33359,
                36895,
                "Current global users 33359, max 36895",
            ))
        )
    }

    #[test]
    fn test_parse_connection_stats() {
        let msg = parse_line(":copper.libera.chat 250 phillipt :Highest connection count: 2367 (2366 clients) (223598 connections received)\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyConnectionStats);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyConnectionStats(ReplyWithNickAndMessageParams::new(
                &Nickname::new("phillipt"),
                "Highest connection count: 2367 (2366 clients) (223598 connections received)",
            ))
        )
    }

    #[test]
    fn test_parse_message_of_the_day_start() {
        let msg = parse_line(":copper.libera.chat 375 phillipt :- copper.libera.chat Message of the Day -\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyMessageOfTheDayStart);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyMessageOfTheDayStart(ReplyWithNickAndMessageParams::new(
                &Nickname::new("phillipt"),
                "- copper.libera.chat Message of the Day -",
            ))
        )
    }

    #[test]
    fn test_parse_message_of_the_day_line() {
        let msg = parse_line(":copper.libera.chat 372 phillipt :- Welcome to Libera Chat, the IRC network for\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyMessageOfTheDayLine);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyMessageOfTheDayLine(ReplyWithNickAndMessageParams::new(
                &Nickname::new("phillipt"),
                "- Welcome to Libera Chat, the IRC network for",
            ))
        )
    }

    #[test]
    fn test_parse_message_of_the_day_end() {
        let msg = parse_line(":copper.libera.chat 376 phillipt :End of /MOTD command.\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ReplyMessageOfTheDayEnd);
        assert_eq!(
            msg.command,
            IrcCommand::ReplyMessageOfTheDayEnd(ReplyWithNickAndMessageParams::new(
                &Nickname::new("phillipt"),
                "End of /MOTD command.",
            ))
        )
    }

    #[test]
    fn test_parse_mode() {
        let msg = parse_line(":phillipt MODE phillipt :+iw\r\n");
        assert_eq!(msg.origin, Some("phillipt".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::Mode);
        assert_eq!(
            msg.command,
            IrcCommand::Mode(ModeParams::new(
                &Nickname::new("phillipt"),
                "+iw",
            ))
        )
    }

    #[test]
    fn test_parse_ping() {
        let msg = parse_line("PING :copper.libera.chat\r\n");
        assert_eq!(msg.origin, None);
        assert_eq!(msg.command_name, IrcCommandName::Ping);
        assert_eq!(
            msg.command,
            IrcCommand::Ping(PingParams::new("copper.libera.chat"))
        )
    }

    #[test]
    fn test_parse_quit() {
        let msg = parse_line(":phillipt!~phillipt@86.11.226.171 QUIT :Ping timeout: 264 seconds\r\n");
        assert_eq!(msg.origin, Some("phillipt!~phillipt@86.11.226.171".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::Quit);
        assert_eq!(
            msg.command,
            IrcCommand::Quit(QuitParams::new("Ping timeout: 264 seconds"))
        )
    }

    #[test]
    fn test_parse_error() {
        let msg = parse_line("ERROR :Closing Link: 86.11.226.171 (Ping timeout: 264 seconds)\r\n");
        assert_eq!(msg.origin, None);
        assert_eq!(msg.command_name, IrcCommandName::Error);
        assert_eq!(
            msg.command,
            IrcCommand::Error(ErrorParams::new("Closing Link: 86.11.226.171 (Ping timeout: 264 seconds)"))
        )
    }

    #[test]
    fn test_parse_error_no_such_nick() {
        let msg = parse_line(":copper.libera.chat 401 user msg :No such nick/channel\r\n");
        assert_eq!(msg.origin, Some("copper.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ErrorNoSuchNick);
        assert_eq!(
            msg.command,
            IrcCommand::ErrorNoSuchNick(DescriptorAndReasonParams::new("user msg", "No such nick/channel"))
        )
    }

    #[test]
    fn test_parse_error_unknown_command() {
        let msg = parse_line(":zirconium.libera.chat 421 test CMD :Unknown command\r\n");
        assert_eq!(msg.origin, Some("zirconium.libera.chat".to_string()));
        assert_eq!(msg.command_name, IrcCommandName::ErrorUnknownCommand);
        assert_eq!(
            msg.command,
            IrcCommand::ErrorUnknownCommand(ErrorUnknownCommandParams::new(Nickname("test".to_string()), "CMD", "Unknown command"))
        )
    }
}
