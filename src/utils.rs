use crate::AbstractSyntaxTree;
use std::fmt::{Debug, Display};

#[derive(PartialEq)]
pub enum PropagatedParseError {
    /// When an atom fails, its error has its propagation field set to `Atomic(actual, expected)`.
    Atomic(String, String),

    /// When a brick was sent to `ParserIndex::RejectingState` because the last parser it tried accepted,
    /// its error has its propagation field set to `BecauseSubparserAccepted(subparser_name, subparser_output)`.
    BecauseSubparserAccepted(String, AbstractSyntaxTree),

    /// When a brick was sent to `ParserIndex::RejectingState` because the last parser it tried failed,
    /// its error has its propagation field set to `BecauseSubparserRejected(subparser_error)`.
    BecauseSubparserRejected(Box<ParseError>),
}

#[derive(PartialEq)]
pub struct ParseError {
    pub message: String,
    pub parser_name: String,
    pub propagation: PropagatedParseError,
}

impl ParseError {
    pub fn new<T: ToString, U: ToString>(
        message: T,
        parser_name: U,
        propagation: PropagatedParseError,
    ) -> Self {
        ParseError {
            message: message.to_string(),
            parser_name: parser_name.to_string(),
            propagation,
        }
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

pub type ParseResult<'a, T> = Result<(&'a str, T), ParseError>;
//                                             ^ Interpreted parsed string
//                                    ^^^^^^^ Remaining output
