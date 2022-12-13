use std::fmt::{self, Display};

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

// This is a bare-bones implementation. A real library would provide additional
// information in its error type, for example the line and column at which the
// error occurred, the byte offset into the input, or the current key being
// processed.
#[derive(Debug)]
pub enum Error {
    // GeneralError
    Message(String),
    // Key Error
    // related Universal Keys or VER Encoding key
    Key(String),
    // Unsupported Length, define by BER encoding rules
    UnsupportedLength(String),
    // write bytes
    IO(std::io::Error),
    // byte encoding
    Encode(String),
    // unmatch type length
    TypeLength(String),
    // content length
    // must has 16 byte or more
    ContentLenght,
    // string encoding error
    ExpectedString,
    // unmatch length between length and content
    ExpectedMapEnd,
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Message(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => formatter.write_str(msg),
            Error::ContentLenght => formatter.write_str("unexpected end of input or less"),
            /* and so forth */
            _ => formatter.write_str("unexpected error"),
        }
    }
}

impl std::error::Error for Error {}
