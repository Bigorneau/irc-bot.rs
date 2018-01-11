use super::BotCmdHandler;
use super::Error;
use super::Module;
use super::Reaction;
use std;
use std::borrow::Cow;
use std::sync::Arc;
use yaml_rust::Yaml;

pub struct BotCommand {
    pub name: Cow<'static, str>,
    pub provider: Arc<Module>,
    pub auth_lvl: BotCmdAuthLvl,
    pub(super) handler: Arc<BotCmdHandler>,
    pub usage_str: Cow<'static, str>,
    pub(super) usage_yaml: Yaml,
    pub help_msg: Cow<'static, str>,
}

#[derive(Debug)]
pub enum BotCmdAttr {}

#[derive(Debug)]
pub enum BotCmdResult {
    /// The command was processed successfully. Pass through a `Reaction`.
    Ok(Reaction),

    /// A user invoked the command without having sufficient authorization to do so. A reply will
    /// be sent informing the user of this.
    Unauthorized,

    /// A user invoked the command with incorrect syntax. A reply will be sent informing the user
    /// of the correct syntax.
    SyntaxErr,

    /// A user invoked the command without providing a required argument, named by the given
    /// string. This is a more specific version of `SyntaxErr` and should be preferred where
    /// applicable.
    ArgMissing(Cow<'static, str>),

    /// A user invoked the command in one-to-one communication (a.k.a. "query" and "PM") without
    /// providing an argument that is required only in one-to-one communication (such as a channel
    /// name, which could normally default to the name of the channel in which the command was
    /// used), named by the given string. This is a more specific version of `ArgMissing` and
    /// should be preferred where applicable.
    ArgMissing1To1(Cow<'static, str>),

    /// Pass through an instance of the framework's `Error` type.
    LibErr(Error),

    /// A user made some miscellaneous error in invoking the command. The given string will be
    /// included in a reply informing the user of their error.
    UserErrMsg(Cow<'static, str>),

    /// A miscellaneous error that doesn't seem to be the user's fault occurred while the bot was
    /// processing the command. The given string will be included in a reply informing the user of
    /// this.
    BotErrMsg(Cow<'static, str>),
}

impl From<Reaction> for BotCmdResult {
    fn from(r: Reaction) -> Self {
        BotCmdResult::Ok(r)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BotCmdAuthLvl {
    Public,
    Admin,
}

pub fn parse_arg<'s>(syntax: &'s Yaml, arg_str: &str) -> std::result::Result<Yaml, BotCmdResult> {
    use util::yaml as uy;

    match uy::parse_and_check_node(
        arg_str,
        syntax,
        "<argument>",
        || Yaml::Hash(Default::default()),
    ) {
        Ok(arg) => Ok(arg),
        Err(uy::Error(uy::ErrorKind::YamlScan(_), _)) => Err(BotCmdResult::SyntaxErr),
        Err(err) => Err(BotCmdResult::LibErr(err.into())),
    }
}
