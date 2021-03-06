use super::ServerId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MsgDest<'a> {
    pub server_id: ServerId,
    pub target: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MsgPrefix<'a> {
    pub nick: Option<&'a str>,
    pub user: Option<&'a str>,
    pub host: Option<&'a str>,
}

#[derive(Debug)]
pub struct MsgMetadata<'a> {
    pub dest: MsgDest<'a>,
    pub prefix: MsgPrefix<'a>,
}

#[derive(Debug)]
pub struct OwningMsgPrefix {
    backing: String,
}

#[cfg(feature = "pircolate")]
fn prefix_from_pircolate<'a>(
    pirc_pfx: Option<(&'a str, Option<&'a str>, Option<&'a str>)>,
) -> MsgPrefix<'a> {
    match pirc_pfx {
        Some((nick, user, host)) => MsgPrefix {
            nick: Some(nick),
            user: user,
            host: host,
        },
        None => MsgPrefix {
            nick: None,
            user: None,
            host: None,
        },
    }
}

pub(super) fn is_msg_to_nick(target: &str, msg: &str, nick: &str) -> bool {
    target == nick || msg == nick || (msg.starts_with(nick) && (msg.find(|c: char| {
        [':', ','].contains(&c)
    }) == Some(nick.len())))
}

pub(super) fn parse_msg_to_nick<'msg>(
    text: &'msg str,
    target: &str,
    nick: &str,
) -> Option<&'msg str> {
    if is_msg_to_nick(target, text, nick) {
        Some(
            text.trim_left_matches(nick)
                .trim_left_matches(|c: char| [':', ','].contains(&c))
                .trim(),
        )
    } else {
        None
    }
}

pub(super) fn parse_prefix(prefix: &str) -> MsgPrefix {
    let mut iter = prefix.rsplitn(2, '@');
    let host = iter.next();
    let mut iter = iter.next().unwrap_or("").splitn(2, '!');
    let nick = iter.next();
    let user = iter.next();
    MsgPrefix {
        nick: nick,
        user: user,
        host: host,
    }
}

impl<'a> MsgPrefix<'a> {
    /// Returns an upper bound on the length of the message prefix, accurate to within a few bytes.
    pub fn len(&self) -> usize {
        fn component_len(component: Option<&str>) -> usize {
            component.map(|s| s.len()).unwrap_or(0)
        }

        component_len(self.nick) + component_len(self.user) + component_len(self.host) + 2
    }

    /// Converts the `MsgPrefix` into an `OwningMsgPrefix`.
    ///
    /// This can't be a `ToOwned` implementation because that would conflict with `MsgPrefix`'s
    /// `Clone` implementation.
    pub fn to_owning(&self) -> OwningMsgPrefix {
        OwningMsgPrefix::from_string(format!(
            "{}!{}@{}",
            self.nick.unwrap_or(""),
            self.user.unwrap_or(""),
            self.host.unwrap_or("")
        ))
    }
}

impl OwningMsgPrefix {
    pub fn from_string(prefix: String) -> Self {
        OwningMsgPrefix { backing: prefix }
    }

    pub fn parse<'a>(&'a self) -> MsgPrefix<'a> {
        parse_prefix(&self.backing)
    }

    /// Returns the exact length of the message prefix.
    pub fn len(&self) -> usize {
        self.backing.len()
    }

    /// Write each non-`None` field of the given message prefix over the corresponding field in
    /// `self`.
    pub(super) fn update_from(&mut self, new: &MsgPrefix) {
        fn updated<'old, 'new>(old: Option<&'old str>, new: Option<&'new str>) -> &'old str
        where
            'new: 'old,
        {
            match (old, new) {
                (_, Some(s)) => s,
                (Some(s), None) => s,
                (None, None) => "",
            }
        }

        self.backing = {
            let old = self.parse();
            format!(
                "{}!{}@{}",
                updated(old.nick, new.nick),
                updated(old.user, new.user),
                updated(old.host, new.host)
            )
        }
    }
}
