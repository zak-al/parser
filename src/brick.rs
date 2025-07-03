use std::rc::Rc;
use crate::parser::{AbstractSyntaxTree, Parser};
use crate::parser::AbstractSyntaxTree::Branch;
use crate::{ParseError, ParseResult};

pub const ACCEPTING_STATE: i32 = -1;
pub const REJECTING_STATE: i32 = -2;

pub struct Brick {
    parsers: Vec<Rc<dyn Parser>>,
    on_success: Vec<i32>,
    on_failure: Vec<i32>,
}

impl Brick {
    pub fn new(parsers: Vec<Rc<dyn Parser>>) -> Brick {
        let n = parsers.len();
        Brick {
            parsers,
            on_success: vec![ACCEPTING_STATE; n],
            on_failure: vec![REJECTING_STATE; n],
        }
    }
}

impl Parser for Brick {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        let mut remaining = input;
        let mut results: Vec<AbstractSyntaxTree> = vec![];
        let mut i: i32 = 0;
        while i >= 0 {
            match self.parsers[i as usize].parse(remaining) {
                Ok((rem, res)) => {
                    remaining = rem;
                    results.push(res);
                    i = self.on_success[i as usize];
                }
                Err(err) => {
                    i = self.on_failure[i as usize];
                }
            }
        }
        if i == ACCEPTING_STATE {
            return Ok((remaining, Branch(results)));
        }
        Err(ParseError::new(format!(
            "Brick failed to parse \"{input}\". Error occurred when trying to parse \"{remaining}\"."
        )))
    }
}