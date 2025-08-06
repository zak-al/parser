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

/// Converter converts a message-passing parser over type T to one over type U, by applying provided conversion functions.
pub struct Converter<T: Copy, U: Copy> {
    parser: Rc<dyn MessagePassingParser<T>>,
    input_converter: fn(U) -> T,
    output_converter: fn(U, T) -> U,
}
