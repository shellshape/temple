use std::process::ExitStatus;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("empty template content")]
    Empty,

    #[error("unclosed quote")]
    UnclosedQuote,

    #[error("missing argument: {0}")]
    ArgumentError(&'static str),

    #[error("unknown template name: {0}")]
    UnknownTemplate(String),

    #[error("unclodes template")]
    UnclosedTemplate,

    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("failed parsing config: {0}")]
    ConfigParseError(#[from] toml::de::Error),

    #[error("unclosed config block")]
    UnclosedConfig,

    #[error("extended template does not have a 'pagecontent' template")]
    ExtendWithNoPageContent,

    #[error("'exec' command failed ({0}): {1}")]
    ExecCommandFailed(ExitStatus, String),

    #[error("'pagecontent' template can not be at a toplevel page file")]
    ToplevelPageContent,
}
