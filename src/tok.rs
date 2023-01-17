use std::fmt::{Debug, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    MoveRight,
    MoveLeft,
    Increment,
    Decrement,
    Output,
    Input,
    LoopStart,
    LoopEnd,
}

impl TokenType {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            '>' => Some(Self::MoveRight),
            '<' => Some(Self::MoveLeft),
            '+' => Some(Self::Increment),
            '-' => Some(Self::Decrement),
            '.' => Some(Self::Output),
            ',' => Some(Self::Input),
            '[' => Some(Self::LoopStart),
            ']' => Some(Self::LoopEnd),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLoc {
    pub line: usize,
    pub col: usize,
}

impl Display for SourceLoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}:{}", self.line, self.col))
    }
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub loc: SourceLoc,
}

#[derive(Debug, Default, Clone)]
pub struct Tokenizer<'a> {
    input: &'a str,
    line: usize,
    col: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            line: 1,
            col: 1,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.input.chars();
        let mut token_type = None;

        for c in chars.by_ref() {
            if let Some(i) = TokenType::from_char(c) {
                token_type = Some(i);
                break;
            }

            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }

        if let Some(token_type) = token_type {
            let token = Token {
                token_type,
                loc: SourceLoc {
                    line: self.line,
                    col: self.col,
                },
            };

            self.input = chars.as_str();
            self.col += 1;

            Some(token)
        } else {
            None
        }
    }
}
