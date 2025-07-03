use crate::{AbstractSyntaxTree, ParseResult, Parser};
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
