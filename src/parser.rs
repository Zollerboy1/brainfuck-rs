use std::{iter::Peekable, num::Wrapping};

use replace_with::replace_with_or_abort_and_return;

use crate::{
    instruction::Instruction,
    tok::{TokenType, Tokenizer},
};

impl Instruction {
    fn parse_move(right: bool, tokenizer: &mut Peekable<Tokenizer>) -> Self {
        let expected = if right {
            TokenType::MoveRight
        } else {
            TokenType::MoveLeft
        };

        let mut amount = 1;
        while tokenizer
            .next_if(|token| token.token_type == expected)
            .is_some()
        {
            amount += 1;
        }

        if right {
            Self::MoveRight { amount }
        } else {
            Self::MoveLeft { amount }
        }
    }

    fn parse_change_cell(increment: bool, tokenizer: &mut Peekable<Tokenizer>) -> Self {
        let expected = if increment {
            TokenType::Increment
        } else {
            TokenType::Decrement
        };

        let mut amount = Wrapping(1u8);
        while tokenizer
            .next_if(|token| token.token_type == expected)
            .is_some()
        {
            amount += 1;
        }

        let amount = amount.0;

        if increment {
            Self::Increment { amount }
        } else {
            Self::Decrement { amount }
        }
    }
}

mod detail {
    use std::{
        error::Error,
        fmt::{Display, Formatter, Result as FmtResult},
        iter::Peekable,
    };

    use replace_with::replace_with_or_abort_and_return;

    use crate::tok::{SourceLoc, TokenType, Tokenizer};

    use super::Instruction;

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub(crate) enum ParseError {
        UnexpectedLoopEnd(SourceLoc),
        ExpectedLoopEnd(SourceLoc),
    }

    impl Display for ParseError {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            match self {
                Self::UnexpectedLoopEnd(loc) => {
                    f.write_fmt(format_args!("unexpected loop end at {}", loc))
                }
                Self::ExpectedLoopEnd(loc) => {
                    f.write_fmt(format_args!("expected loop end for start at {}", loc))
                }
            }
        }
    }

    impl Error for ParseError {}

    pub(crate) struct Parser<'a> {
        pub(crate) tokenizer: Peekable<Tokenizer<'a>>,
        loop_start: Option<SourceLoc>,
    }

    impl<'a> Parser<'a> {
        pub(crate) fn new(tokenizer: Peekable<Tokenizer<'a>>) -> Self {
            Self {
                tokenizer,
                loop_start: None,
            }
        }

        fn new_loop(tokenizer: Peekable<Tokenizer<'a>>, loop_start: SourceLoc) -> Self {
            Self {
                tokenizer,
                loop_start: Some(loop_start),
            }
        }
    }

    impl<'a> Iterator for Parser<'a> {
        type Item = Result<Instruction, ParseError>;

        fn next(&mut self) -> Option<Self::Item> {
            let token = if let Some(token) = self.tokenizer.next() {
                token
            } else if let Some(loop_start) = self.loop_start {
                return Some(Err(ParseError::ExpectedLoopEnd(loop_start)));
            } else {
                return None;
            };

            match token.token_type {
                TokenType::MoveRight => {
                    Some(Ok(Instruction::parse_move(true, &mut self.tokenizer)))
                }
                TokenType::MoveLeft => {
                    Some(Ok(Instruction::parse_move(false, &mut self.tokenizer)))
                }
                TokenType::Increment => Some(Ok(Instruction::parse_change_cell(
                    true,
                    &mut self.tokenizer,
                ))),
                TokenType::Decrement => Some(Ok(Instruction::parse_change_cell(
                    false,
                    &mut self.tokenizer,
                ))),
                TokenType::Output => Some(Ok(Instruction::Output)),
                TokenType::Input => Some(Ok(Instruction::Input)),
                TokenType::LoopStart => {
                    replace_with_or_abort_and_return(&mut self.tokenizer, |tokenizer| {
                        let mut loop_parser = Parser::new_loop(tokenizer, token.loc);

                        let loop_instructions = (&mut loop_parser).collect::<Result<Vec<_>, _>>();

                        (
                            Some(
                                loop_instructions
                                    .map(|instructions| Instruction::Loop { instructions }),
                            ),
                            loop_parser.tokenizer,
                        )
                    })
                }
                TokenType::LoopEnd => {
                    if self.loop_start.is_some() {
                        None
                    } else {
                        Some(Err(ParseError::UnexpectedLoopEnd(token.loc)))
                    }
                }
            }
        }
    }
}

pub struct Parser<'a> {
    tokenizer: Peekable<Tokenizer<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(tokenizer: Tokenizer<'a>) -> Self {
        Self {
            tokenizer: tokenizer.peekable(),
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Instruction;

    fn next(&mut self) -> Option<Instruction> {
        replace_with_or_abort_and_return(&mut self.tokenizer, |tokenizer| {
            let mut parser = detail::Parser::new(tokenizer);

            let instruction = parser.next();

            (instruction.map(Result::unwrap), parser.tokenizer)
        })
    }
}
