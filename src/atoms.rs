use crate::PropagatedParseError::Atomic;
use crate::parser::AbstractSyntaxTree::{Ignore, Leaf};
use crate::parser::*;
use crate::utils::*;
use std::collections::HashSet;
use std::rc::Rc;

pub struct StringParser {
    name: String,
    string: String,
}

impl Parser for StringParser {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        let mut input_it = input.chars();
        let mut pattern_it = self.string.chars();
        while let Some(expected) = pattern_it.next() {
            if let Some(actual) = input_it.next() {
                if actual != expected {
                    return Err(ParseError::new(
                        format!("expected \"{expected}\" but got \"{actual}\""),
                        self.name.clone(),
                        Atomic(actual.to_string(), expected.to_string()),
                    ));
                }
            } else {
                return Err(ParseError::new(
                    format!("exhausted input but expected \"{expected}\""),
                    self.name.clone(),
                    Atomic("".to_string(), expected.to_string()),
                ));
            }
        }
        Ok((
            input_it.as_str(),
            Leaf(self.name.clone(), self.string.clone()),
        ))
    }

    fn get_name_clone(&self) -> String {
        self.name.clone()
    }
}

impl StringParser {
    pub fn new<T: ToString>(string: T) -> StringParser {
        StringParser {
            name: string.to_string(),
            string: string.to_string(),
        }
    }
}

pub struct CharacterFromAlphabet {
    pub(crate) name: String,
    pub alphabet: Rc<HashSet<char>>,
}

impl Parser for CharacterFromAlphabet {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        let mut input_it = input.chars();
        match input_it.next() {
            None => Err(ParseError::new(
                format!(
                    "Expected character from alphabet {:?} but found an empty input.",
                    self.alphabet
                ),
                self.name.clone(),
                Atomic("".to_string(), format!("{:?}", self.alphabet)),
            )),
            Some(c) => {
                if self.alphabet.contains(&c) {
                    return Ok((input_it.as_str(), Leaf(self.name.clone(), c.to_string())));
                }
                Err(ParseError::new(
                    format!(
                        "Expected character from alphabet {:?} but found {c}.",
                        self.alphabet
                    ),
                    self.name.clone(),
                    Atomic(c.to_string(), format!("{:?}", self.alphabet)),
                ))
            }
        }
    }

    fn get_name_clone(&self) -> String {
        self.name.clone()
    }
}

/// Parses any word over the specified alphabet.
pub struct AllWordsFromAlphabet {
    pub name: String,
    pub alphabet: Rc<HashSet<char>>,
    pub allow_empty_word: bool,
}

impl Parser for AllWordsFromAlphabet {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        let end = input
            .char_indices()
            .position(|(_, c)| !self.alphabet.contains(&c));
        match end {
            // If all characters in the input are in the alphabet, we still need to check whether
            // the input is empty.
            None => {
                if !input.is_empty() || self.allow_empty_word {
                    Ok(("", Leaf(self.name.clone(), input.to_string())))
                } else {
                    Err(ParseError::new(
                        format!(
                            "Expected a non-empty word over alphabet {:?} but got nothing.",
                            self.alphabet
                        ),
                        self.name.clone(),
                        Atomic("".to_string(), format!("non-empty of {:?}", self.alphabet)),
                    ))
                }
            }
            Some(end) => {
                // We split the input around the index of the first character that we cannot parse.
                let (parsed, remaining) = input.split_at(end);

                if !parsed.is_empty() || self.allow_empty_word {
                    Ok((remaining, Leaf(self.name.clone(), parsed.to_string())))
                } else {
                    let actual = match remaining.chars().next() {
                        None => "nothing".to_string(),
                        Some(x) => x.to_string(),
                    };
                    Err(ParseError::new(
                        format!(
                            "Expected a non-empty word over alphabet {:?} but got \"{}\".",
                            self.alphabet, actual
                        ),
                        self.name.clone(),
                        Atomic(actual, format!("non-empty of {:?}", self.alphabet)),
                    ))
                }
            }
        }
    }

    fn get_name_clone(&self) -> String {
        self.name.clone()
    }
}

pub struct EndOfInputParser;
impl Parser for EndOfInputParser {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        if input.is_empty() {
            return Ok((input, Ignore));
        }
        Err(ParseError::new(
            format!("Expected end of input but got \"{}\"", input),
            "end_of_input",
            Atomic(input.to_string(), "".to_string()),
        ))
    }

    fn get_name_clone(&self) -> String {
        "end_of_input".to_string()
    }
}

