use code::Code;
use failure::Error;

/// Error generated by the parser.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Fail)]
pub enum ParseError {
    /// String was empty.
    #[fail(display = "String was empty.")]
    EmptyCommand,
    /// Message did not have a code.
    #[fail(display = "Message did not have a code.")]
    EmptyMessage,
    /// Unexpected end of the string.
    #[fail(display = "Unexpected end of the string.")]
    UnexpectedEnd,
}

/// Represents a message received from the server.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    /// Prefix
    pub prefix: Option<Prefix>,
    /// Code
    pub code: Code,
    /// Arguments
    pub args: Vec<String>,
}

impl Message {
    /// Parse the given string into a `Message` struct.
    ///
    /// An error is returned if the message is not valid.
    pub fn parse(line: &str) -> Result<Message, Error> {
        if line.is_empty() || line.trim().is_empty() {
            return Err(ParseError::EmptyMessage.into());
        }

        let mut state = line.trim_right_matches("\r\n");
        let mut prefix: Option<Prefix> = None;
        let code: Option<&str>;
        let mut args: Vec<String> = Vec::new();

        // Look for a prefix
        if state.starts_with(':') {
            match state.find(' ') {
                None => return Err(ParseError::UnexpectedEnd.into()),
                Some(idx) => {
                    prefix = parse_prefix(&state[1..idx]);
                    state = &state[idx + 1..];
                }
            }
        }

        // Look for the command/reply
        match state.find(' ') {
            None => {
                if state.is_empty() {
                    return Err(ParseError::EmptyMessage.into());
                } else {
                    code = Some(&state[..]);
                    state = &state[state.len()..];
                }
            }
            Some(idx) => {
                code = Some(state[..idx].into());
                state = &state[idx + 1..];
            }
        }

        // Look for arguments and the suffix
        if !state.is_empty() {
            loop {
                if state.starts_with(':') {
                    args.push(state[1..].into());
                    break;
                } else {
                    match state.find(' ') {
                        None => {
                            args.push(state[..].into());
                            break;
                        }
                        Some(idx) => {
                            args.push(state[..idx].into());
                            state = &state[idx + 1..];
                        }
                    }
                }
            }
        }

        let code = match code {
            None => return Err(ParseError::EmptyCommand.into()),
            Some(text) => match text.parse() {
                Ok(code) => code,
                Err(_) => Code::Unknown(text.into()),
            },
        };

        Ok(Message { prefix, code, args })
    }
}

fn parse_prefix(prefix: &str) -> Option<Prefix> {
    match prefix.find('!') {
        None => Some(Prefix::Server(prefix.to_string())),
        Some(excpos) => {
            let nick = &prefix[..excpos];
            let rest = &prefix[excpos + 1..];
            match rest.find('@') {
                None => None,
                Some(atpos) => {
                    let user = &rest[..atpos];
                    let host = &rest[atpos + 1..];
                    Some(Prefix::User(PrefixUser::new(nick, user, host)))
                }
            }
        }
    }
}

/// Prefix of the message.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Prefix {
    /// Prefix is a user.
    User(PrefixUser),
    /// Prefix is a server.
    Server(String),
}

/// User prefix representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefixUser {
    /// Nickname
    pub nickname: String,
    /// Username
    pub username: String,
    /// Hostname
    pub hostname: String,
}

impl PrefixUser {
    fn new(nick: &str, user: &str, host: &str) -> PrefixUser {
        PrefixUser {
            nickname: nick.into(),
            username: user.into(),
            hostname: host.into(),
        }
    }
}

#[test]
fn test_full() {
    let res = Message::parse(":org.prefix.cool COMMAND arg1 arg2 arg3 :suffix is pretty cool yo");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(msg.code, Code::Unknown("COMMAND".to_string()));
    assert_eq!(
        msg.args,
        vec!["arg1", "arg2", "arg3", "suffix is pretty cool yo"]
    );
}

#[test]
fn test_no_prefix() {
    let res = Message::parse("NICK arg1 arg2 arg3 :suffix is pretty cool yo");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(msg.prefix, None);
    assert_eq!(msg.code, Code::Nick);
    assert_eq!(
        msg.args,
        vec!["arg1", "arg2", "arg3", "suffix is pretty cool yo"]
    );
}

#[test]
fn test_no_suffix() {
    let res = Message::parse(":org.prefix.cool NICK arg1 arg2 arg3");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(msg.code, Code::Nick);
    assert_eq!(msg.args, vec!["arg1", "arg2", "arg3"]);
}

#[test]
fn test_no_args() {
    let res = Message::parse(":org.prefix.cool NICK :suffix is pretty cool yo");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(msg.code, Code::Nick);
    assert_eq!(msg.args, vec!["suffix is pretty cool yo"]);
}

#[test]
fn test_only_command() {
    let res = Message::parse("NICK");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(msg.prefix, None);
    assert_eq!(msg.code, Code::Nick);
    assert_eq!(msg.args.len(), 0);
}

#[test]
fn test_empty_message() {
    let res = Message::parse("");
    assert!(res.is_err());
    let err: ParseError = res.err().unwrap().downcast().unwrap();
    assert!(err == ParseError::EmptyMessage);
}

#[test]
fn test_empty_message_trim() {
    let res = Message::parse("    ");
    assert!(res.is_err());
    let err: ParseError = res.err().unwrap().downcast().unwrap();
    assert!(err == ParseError::EmptyMessage);
}

#[test]
fn test_only_prefix() {
    let res = Message::parse(":org.prefix.cool");
    assert!(res.is_err());
    let err: ParseError = res.err().unwrap().downcast().unwrap();
    assert!(err == ParseError::UnexpectedEnd);
}

#[test]
fn test_prefix_none() {
    let res = Message::parse("COMMAND :suffix is pretty cool yo");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(msg.args, vec!["suffix is pretty cool yo"]);
}

#[test]
fn test_prefix_server() {
    let res = Message::parse(":irc.freenode.net COMMAND :suffix is pretty cool yo");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(msg.prefix, Some(Prefix::Server("irc.freenode.net".into())));
}

#[test]
fn test_prefix_user() {
    let res = Message::parse(":bob!bob@bob.com COMMAND :suffix is pretty cool yo");
    assert!(res.is_ok());
    let msg = res.ok().unwrap();
    assert_eq!(
        msg.prefix,
        Some(Prefix::User(PrefixUser::new("bob", "bob", "bob.com")))
    );
}
