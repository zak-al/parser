use crate::{AbstractSyntaxTree, MessagePassingParser, ParseResult, Parser};
use std::rc::Rc;

/// Ignore is a parser that applies the wrapped parser.
/// If the wrapped parser fails, Ignore fails and propagate the error of the wrapped parser.
/// If the wrapped parser accepts, Ignore accepts and outputs an Ignore leaf.
/// This is useful for elements that need to be parsed but are not used when processing the abstract syntax tree, like whitespaces and trailing commas.
pub struct Ignore {
    name: String,
    parser: Rc<dyn Parser>,
}

impl Parser for Ignore {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        match self.parser.parse(input) {
            Ok(_) => Ok((input, AbstractSyntaxTree::Ignore)),
            Err(e) => Err(e),
        }
    }

    fn get_name_clone(&self) -> String {
        self.name.clone()
    }
}

impl Ignore {
    pub fn new(parser: Rc<dyn Parser>) -> Ignore {
        Ignore {
            name: format!("ignore_{}", parser.get_name_clone()),
            parser,
        }
    }
}

/// IdentityMessenger is a wrapper around a regular parser that makes it a message-passing parser
/// that simply takes a parameter of any type and returns it.
pub struct IdentityMessenger {
    parser: Rc<dyn Parser>,
}

impl<T: Copy> MessagePassingParser<T> for IdentityMessenger {
    fn parse_with_message<'a>(&self, input: &'a str, parameter: T) -> (ParseResult<'a, AbstractSyntaxTree>, T) {
        (self.parser.parse(input), parameter)
    }

    fn get_name_clone(&self) -> String {
        self.parser.get_name_clone()
    }
}

impl IdentityMessenger {
    pub fn new(parser: Rc<dyn Parser>) -> IdentityMessenger {
        IdentityMessenger {parser}
    }
}

/// Converter converts a message-passing parser over type T to one over type U, by applying provided conversion functions.
pub struct Converter<T: Copy, U: Copy> {
    pub(crate) parser: Rc<dyn MessagePassingParser<T>>,
    pub(crate) input_converter: fn(U) -> T,
    //                                   ^ parameter sent to the wrapped parser
    //                             ^ parameter sent to the converted parser
    pub(crate) output_converter: fn(U, T) -> U,
}

impl <T: Copy, U: Copy> MessagePassingParser<U> for Converter<T, U> {
    fn parse_with_message<'a>(&self, input: &'a str, parameter: U) -> (ParseResult<'a, AbstractSyntaxTree>, U) {
        let (res, msg) = self.parser.parse_with_message(input, (self.input_converter)(parameter));
        (res, (self.output_converter)(parameter, msg))
    }

    fn get_name_clone(&self) -> String {
        self.parser.get_name_clone()
    }
}

pub struct SimpleParser<T: Copy> {
    parser: Rc<dyn MessagePassingParser<T>>,
    default_value: T,
}

impl<T: Copy> Parser for SimpleParser<T> {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        let (res, _) = self.parser.parse_with_message(input, self.default_value);
        res
    }

    fn get_name_clone(&self) -> String {
        self.parser.get_name_clone()
    }
}

impl<T: Copy> SimpleParser<T> {
    fn new(parser: Rc<dyn MessagePassingParser<T>>, default_value: T) -> SimpleParser<T> {
        SimpleParser {
            parser,
            default_value
        }
    }
}
