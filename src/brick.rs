use crate::PropagatedParseError::{BecauseSubparserAccepted, BecauseSubparserRejected};
use crate::brick::ParserIndex::{AcceptingState, Index, RejectingState};
use crate::parser::AbstractSyntaxTree::{Branch, Ignore};
use crate::parser::{AbstractSyntaxTree, Parser};
use crate::{ParseError, ParseResult, ignore};
use std::rc::Rc;

#[derive(Debug, Clone, Copy)]
pub enum ParserIndex {
    Index(usize),
    AcceptingState,
    RejectingState,
}

impl From<usize> for ParserIndex {
    fn from(index: usize) -> Self {
        Index(index)
    }
}

/// A brick is a parser that combines other parsers.
/// It contains a vector of parsers and two vectors, `on_success` and `on_failure` which describe
/// the behaviour of the brick when each parser accepts or refuses its input.
/// When calling Brick::parse on some input, `parsers[0].parse` will be called with the provided input,
/// and subsequent parsers will be called according to the dynamics specified in `on_success` and `on_failure`.
pub struct Brick {
    name: String,
    parsers: Vec<Rc<dyn Parser>>,

    /// `on_success` must be the same size as `parsers`. `on_success[i]` can be:
    /// - an index `ParserIndex::Index(j)`, which means that when `parsers[i]` accepts its input,
    /// the remaining input will be sent to `parsers[j]`;
    /// - the token `ParserIndex::AcceptingState`, which means that when `parsers[i]` accepts its input,
    /// the brick will accept the input.
    /// - the token `ParserIndex::RejectingState`, which means that when `parsers[i]` rejects its input,
    /// the brick will reject the input.
    pub on_success: Vec<ParserIndex>,

    /// `on_failure` works like `on_success`, determining what parser will be run when a parser rejects its input.
    pub on_failure: Vec<ParserIndex>,
}

impl Brick {
    /// `Brick::new` defines a new brick from a list of parsers, with default values for on_success and on_failure.
    /// The default behaviour makes the brick equivalent to the first of the provided parsers, whose result is wrapped in a AbstractSyntaxTree::branch.
    pub fn new<T: ToString>(name: T, parsers: Vec<Rc<dyn Parser>>) -> Brick {
        let n = parsers.len();
        if n == 0 {
            panic!("Tried to create a brick with no parsers. This is illegal.");
        }
        Brick {
            name: name.to_string(),
            parsers,
            on_success: vec![AcceptingState; n],
            on_failure: vec![RejectingState; n],
        }
    }

    pub fn make_linear<T: ToString>(name: T, parsers: Vec<Rc<dyn Parser>>) -> Brick {
        let n = parsers.len();
        if n == 0 {
            panic!("Tried to create a brick with no parsers. This is illegal.");
        }
        let mut on_success: Vec<ParserIndex> = (1..=n).map(ParserIndex::from).collect();
        on_success[n - 1] = AcceptingState;
        Brick {
            name: name.to_string(),
            parsers,
            on_success,
            on_failure: vec![RejectingState; n],
        }
    }

    /// Runs all parsers in the order in which they are given until one accepts.
    /// Accepts the input if a parser accepts, rejects the input if none accepts.
    pub fn make_disjunction<T: ToString>(name: T, parsers: Vec<Rc<dyn Parser>>) -> Brick {
        let n = parsers.len();
        if n == 0 {
            panic!("Tried to create a brick with no parsers. This is illegal.");
        }
        let mut on_failure: Vec<ParserIndex> = (1..=n).map(ParserIndex::from).collect();
        on_failure[n - 1] = RejectingState;
        Brick {
            name: name.to_string(),
            parsers,
            on_success: vec![AcceptingState; n],
            on_failure,
        }
    }

    /// Applies the parser and accepts the input. The input is consumed if and only if the parser accepted.
    /// It propagates the output of the wrapped parser, unlike Ignore.
    /// Can be wrapped inside an Ignore to optionally consume an element that has no effect on the meaning of the input.
    pub fn maybe<T: ToString>(name: T, parser: Rc<dyn Parser>) -> Brick {
        Brick {
            name: name.to_string(),
            parsers: vec![parser],
            on_success: vec![AcceptingState],
            on_failure: vec![AcceptingState],
        }
    }

    /// Makes a brick that parses a chain that contains elements matched by parser, seperated by elements matched by separator.
    /// parser must match at least one element. It may or may not be allowed to match strings that end with the separator, by setting the allow_trailing_separator attribute accordingly. Even if we allow trailing separator, there must be at least one element parsed (internally, this works by parsing a separated string without trailing separator and then optionally parsing a separator).
    /// For example, if parser matches strings of numbers and separator matches operators, this will match operations.
    pub fn make_separated<T: ToString>(
        name: T,
        parser: Rc<dyn Parser>,
        separator: Rc<dyn Parser>,
        allow_trailing_separator: bool,
    ) -> Brick {
        if !allow_trailing_separator {
            Brick {
                name: name.to_string(),
                parsers: vec![parser, separator],
                on_success: vec![Index(1), Index(0)],
                on_failure: vec![RejectingState, AcceptingState],
            }
        } else {
            let trailing_separator = Rc::new(ignore::Ignore::new(Rc::new(Brick::maybe(
                format!("{}_trailing_separator", name.to_string()),
                separator.clone(),
            ))));
            Brick::make_linear(name, vec![parser, separator, trailing_separator])
        }
    }
}

impl Parser for Brick {
    fn parse<'a>(&self, input: &'a str) -> ParseResult<'a, AbstractSyntaxTree> {
        let name = self.name.clone();
        let mut remaining = input;
        let mut results: Vec<AbstractSyntaxTree> = vec![];
        let mut i = Index(0);
        let mut last_parser_index: Option<usize> = None;
        let mut last_parser_accepted: bool = false;
        let mut last_failure: Option<ParseError> = None;
        while let Index(j) = i {
            last_parser_index = Some(j);
            match self.parsers[j].parse(remaining) {
                Ok((rem, res)) => {
                    remaining = rem;
                    if res != Ignore {
                        results.push(res);
                    }
                    i = self.on_success[j];
                    last_parser_accepted = true;
                }
                Err(err) => {
                    i = self.on_failure[j];
                    last_parser_accepted = false;
                    last_failure = Some(err);
                }
            }
        }
        match i {
            AcceptingState => Ok((remaining, Branch(name, results))),
            RejectingState => {
                let propagation = if last_parser_accepted {
                    let last_parser_name = self.parsers
                        [last_parser_index.expect("Internal error. Please report.")]
                    .get_name_clone();
                    BecauseSubparserAccepted(
                        last_parser_name,
                        results.pop().expect("Internal error. Please report."),
                    )
                } else {
                    BecauseSubparserRejected(Box::new(
                        last_failure.expect("Internal error. Please report."),
                    ))
                };
                Err(ParseError::new(
                    format!(
                        "Brick failed to parse \"{input}\". Error occurred when trying to parse \"{remaining}\"."
                    ),
                    self.name.clone(),
                    propagation,
                ))
            }
            _ => {
                panic!("Internal error. Please report.")
            }
        }
    }
    fn get_name_clone(&self) -> String {
        self.name.clone()
    }
}
