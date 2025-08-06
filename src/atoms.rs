use crate::PropagatedParseError::Atomic;
use crate::parser::AbstractSyntaxTree::Leaf;
use crate::parser::*;
use crate::utils::*;
use std::collections::HashSet;
use std::rc::Rc;
use crate::AbstractSyntaxTree::Ignore;
use crate::atoms::LengthParameter::Length;

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

impl<T: Copy> MessagePassingParser<T> for StringParser {
    fn parse_with_message<'a>(&self, input: &'a str, parameter: T) -> (ParseResult<'a, AbstractSyntaxTree>, T) {
        (self.parse(input), parameter)
    }
    fn get_name_clone(&self) -> String {
        Parser::get_name_clone(self)
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

#[derive(Copy, Clone)]
pub enum LengthParameter {
    Length(usize),
    Any
}

impl MessagePassingParser<LengthParameter> for AllWordsFromAlphabet {
    fn parse_with_message<'a>(
        &self,
        input: &'a str,
        parameter: LengthParameter,
    ) -> (ParseResult<'a, AbstractSyntaxTree>, LengthParameter) {
        match parameter {
            Length(parameter) => {
                // We look for the first character not to parse.
                let end = input
                    .char_indices()
                    .position(|(i, x)| i >= parameter || !self.alphabet.contains(&x));
                //                                 ^ the string must end at index i-1 at most
                //                                                   ^ the string must not contain
                //                                                     characters not in self.alphabet.
                match end {
                    None => (
                        // This means that the input contains only characters from the alphabet, but too few.
                        Err(ParseError::new(
                            format!(
                                "Expected string of length {} over {:?} but got input {}.",
                                parameter, self.alphabet, input
                            ),
                            self.name.clone(),
                            Atomic(input.to_string(), format!("{:*<w$}", input, w=parameter)),
                        )),
                        Length(parameter),
                    ),
                    Some(end) => {
                        // This means that input[0..=end-1] is of length at most parameter and contains only characters from self.alphabet.
                        let (parsed, remaining) = input.split_at(end);
                        if end == parameter {
                            (Ok((remaining, Leaf(self.name.clone(), parsed.to_string()))), Length(parameter))
                        } else {
                            (Err(ParseError::new(format!("Expected {} characters from {:?} but the longest string from the alphabet is {}, which has length {}", parameter, self.alphabet, parsed, end), self.name.clone(), Atomic(parsed.to_string(), format!("{:*<w$}", parsed, w=parameter)))), Length(parameter))
                        }
                    }
                }
            },
            LengthParameter::Any => {
                match self.parse(input) {
                    Ok((rem, Leaf(tok, parsed))) => {
                        let n = parsed.len();
                        (Ok((rem, Leaf(tok, parsed))), Length(n))
                    },
                    Err(err) => {
                        (Err(err), Length(0))
                    },
                    Ok(_) => {
                        panic!("Internal error. Please report.")
                    }
                }
            }
        }

    }

    fn get_name_clone(&self) -> String {
        Parser::get_name_clone(self)
    }
}

pub struct EndOfInputParser;
impl Parser for EndOfInputParser {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        if input.is_empty() {
            return Ok((input, AbstractSyntaxTree::Ignore));
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


pub struct MessageTransformer<T: Copy> {
    transformation: dyn Fn(T) -> T,
}

impl<T: Copy> MessagePassingParser<T> for MessageTransformer<T> {
    fn parse_with_message<'a>(&self, input: &'a str, parameter: T) -> (ParseResult<'a, AbstractSyntaxTree>, T) {
        (Ok((input, Ignore)), (self.transformation)(parameter))
    }

    fn get_name_clone(&self) -> String {
        "".to_string()
    }
}
