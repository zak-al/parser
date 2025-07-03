use crate::ParseResult;

pub trait Parser {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree>;
    fn get_name_clone(&self) -> String;
}

#[derive(Debug, PartialEq, Eq)]
pub enum AbstractSyntaxTree {
    Leaf(String, String),
    Branch(String, Vec<AbstractSyntaxTree>),
    Ignore,
}
