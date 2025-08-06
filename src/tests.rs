#[cfg(test)]
mod tests {
    use crate::atoms::*;
    use crate::brick::ParserIndex::{AcceptingState, Index, RejectingState};
    use crate::brick::{Brick, ParserIndex};
    use crate::{MessagePassingBrick, NUMERICS, Parser};
    use std::collections::HashSet;
    use std::rc::Rc;
    use crate::combinators::Converter;
    use crate::LengthParameter::{Any, Length};

    #[test]
    fn test_parse_all_words_from_alphabet() {
        let parse_numbers = Rc::new(AllWordsFromAlphabet {
            name: "numerics".to_string(),
            alphabet: NUMERICS.clone(),
            allow_empty_word: false,
        });
        assert!(parse_numbers.parse("123").is_ok());
        assert!(parse_numbers.parse("+").is_err());
        assert!(parse_numbers.parse("").is_err());
    }

    #[test]
    fn test_expression() {
        // In this test, we consider a simplified version of Python's rule "Expression".
        // An expression is recursively defined as follows:
        // Expression: "disjunction" "if" "disjunction" "else" Expression
        //           | "disjunction"
        //           | "lambda"
        // where a string in quote indicates perfect match, juxtaposition is composition,
        // and | indicates disjunction.

        // We first define atomic parsers:
        let disjunction_string = Rc::new(StringParser::new("disjunction"));
        let lambda = Rc::new(StringParser::new("lambda"));
        let _if = Rc::new(StringParser::new(" if "));
        let _else = Rc::new(StringParser::new(" else "));

        // There are three bricks to an Expresion parser: the "disjunction if disjunction else" chain,
        // the "disjunction" string and the "lambda" string.
        // "disjunction" and "lambda" were already defined. A single object suffices.
        let chain = Rc::new(Brick::make_linear(
            "ternary_operator",
            vec![
                disjunction_string.clone(),
                _if.clone(),
                disjunction_string.clone(),
                _else.clone(),
            ],
        ));
        let mut expression = Brick::new(
            "expression",
            vec![chain.clone(), disjunction_string.clone(), lambda.clone()],
        );

        // We now need to tell the expression brick how these three parsers interact.
        // If we successfully parsed the chain, we're not done yet: we now need to parse a new expression.
        expression.on_success[0] = ParserIndex::Index(0);
        // If we fail to parse the chain, it's fine: maybe the expression is just the string "disjunction".
        expression.on_failure[0] = ParserIndex::Index(1);
        // If we successfully parse "disjunction", we're done! Otherwise, we try to parse "lambda".
        expression.on_success[1] = AcceptingState;
        expression.on_failure[1] = ParserIndex::Index(2);
        // Since lambda is the last member of the disjunction that defines expressions that we try,
        // we accept or reject the expression if and only if lambda accepts or rejects.
        expression.on_success[2] = AcceptingState;
        expression.on_failure[2] = RejectingState;

        // We define a parser that tries to parse a string that starts with an expression and rejects otherwise.
        let expression = Rc::new(expression);
        let eof = Rc::new(EndOfInputParser);
        let mut parser = Brick::new("only_expression", vec![expression.clone(), eof.clone()]);
        parser.on_success[0] = Index(1); // if we parse an expression, we'll look for end-of-input.
        parser.on_failure[0] = RejectingState;
        // eof is run if and only if an expression was successfully parsed: whether it accepts
        // determines whether we accept the input.
        parser.on_success[1] = AcceptingState;
        parser.on_failure[1] = RejectingState;

        assert!(parser.parse("lambda").is_ok());
        assert!(parser.parse("disjunction").is_ok());
        assert!(parser.parse("if").is_err());
        assert!(parser.parse("else").is_err());
        assert!(parser.parse("").is_err());
        assert!(
            parser
                .parse("disjunction if disjunction else lambda")
                .is_ok()
        );
        assert!(
            parser
                .parse("disjunction if disjunction else disjunction if disjunction else lambda")
                .is_ok()
        );
        assert!(
            parser
                .parse(
                    "disjunction if disjunction else disjunction if disjunction else disjunction"
                )
                .is_ok()
        );
        assert!(parser.parse("disjunction if disjunction else").is_err());
    }

    #[test]
    fn test_chain_of_operations() {
        // This tests creates a parser that matches operations like 123, 123+456, and 123+456/789.
        let parse_numbers = Rc::new(AllWordsFromAlphabet {
            name: "numerics".to_string(),
            alphabet: NUMERICS.clone(),
            allow_empty_word: false,
        });
        let operators: Rc<HashSet<char>> = Rc::new(HashSet::from_iter("+-*/".chars()));
        let parse_operator = Rc::new(CharacterFromAlphabet {
            name: "operator".to_string(),
            alphabet: operators,
        });
        let parse_operation = Rc::new(Brick::make_separated(
            "operation".to_string(),
            parse_numbers.clone(),
            parse_operator.clone(),
            false,
        ));
        let eof = Rc::new(EndOfInputParser);
        let parser =
            Brick::make_linear("only_operation", vec![parse_operation.clone(), eof.clone()]);

        assert!(parser.parse("").is_err());
        assert!(parser.parse("+").is_err());
        assert!(parser.parse("123").is_ok());
        assert!(parser.parse("123+").is_err());
        assert!(parser.parse("123+456").is_ok());
        assert!(parser.parse("0/0").is_ok());
        assert!(parser.parse("0/2*2").is_ok());
        assert!(parser.parse("1").is_ok());
        assert!(parser.parse("12+21+12/12-65").is_ok());
        assert!(parser.parse("+12").is_err());
        assert!(parser.parse("-12").is_err());
        assert!(parser.parse("/").is_err());
    }

    #[test]
    fn test_indented_blocks() {
        // An indented block is a message passing brick that passes messages of type (int, int).
        // The first component of messages is the indentation level and the second is the number of spaces per indentation level (e.g. 2 or 4).
        // Each line has to contain the string "instruction" or "def instruction", and there must be at least one line.
        // After "def instruction", we enter a new indented bock with its level incremented by one.

        // In other words, an indented block with parameter (t, k) parses:
        //  if k != 0 or t = k = 0, t * k spaces; if k = 0 and t > 0,
        //      any number of spaces and set k to the number of spaces parsed
        //  a mandatory "instruction" or "def instruction"
        //  on success of mandatory "def instruction": 0 with parameter (t + 1, k)
        //  on success of mandatory "instruction", either of:
        //          | 0 without changing the parameter
        //          | if t !=, 0 with parameter (t - 1, k)
        //          | eof

        // parametrised_indentation is parametrised by the expected number of indentations
        let parametrised_indentation = Rc::new(AllWordsFromAlphabet {
            name: "indentation".to_string(),
            alphabet: Rc::new(HashSet::from_iter(" ".chars())), // just space
            allow_empty_word: true,
        });
        
        fn get_number_of_spaces((indentation_level, number_of_spaces): (usize, LengthParameter)) -> LengthParameter{
            match number_of_spaces {
                Length(k) => {Length(indentation_level * k)}
                LengthParameter::Any => {LengthParameter::Any}
            }
        }
        let parametrised_indentation = Converter {
            parser: parametrised_indentation,
            input_converter: get_number_of_spaces,
            
        }
        let parametrised_indentation = Rc::new(parametrised_indentation);

        let instruction = Rc::new(StringParser::new("instruction"));
        
        let mut indented_instruction = MessagePassingBrick::new(
            "indented_instruction",
            vec![parametrised_indentation.clone(), instruction.clone()],
        );
        let indented_instruction = Rc::new(indented_instruction);

        let def_instruction = Rc::new(StringParser::new("def instruction"));
        let mut indented_def_instruction = MessagePassingBrick::new(
            "indented_def_instruction",
            vec![parametrised_indentation.clone(), def_instruction.clone()],
        );
        let indented_def_instruction = Rc::new(indented_def_instruction);

        let mut indented_block = MessagePassingBrick::new(
            "indented_block",
            vec![
                indented_instruction.clone(), // mandatory instruction
                indented_def_instruction.clone(), // mandatory def instruction
                indented_instruction.clone(), // optional instruction
                indented_def_instruction.clone(), // optional def instruction
            ],
        );
        indented_block.on_success[0] = Index(0); // After an indented instruction, we stay in the same block.
        indented_block.on_failure[0] = Index(1); // If a block doesn't start with an indented instruction, it must start with a def instruction.
        indented_block.on_success[1] = Index(2);
        indented_block.on_failure[1] = RejectingState;
    }
}
