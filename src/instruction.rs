use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone, PartialEq, Eq)]
pub enum Instruction {
    MoveRight(usize),
    MoveLeft(usize),
    Increment(u8),
    Decrement(u8),
    Output,
    Input,
    Loop(Vec<Instruction>),
    MoveRightUntilZero(usize),
    MoveLeftUntilZero(usize),
    SetToZero,
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::MoveRight(amount) => f.write_fmt(format_args!("MoveRight({})", amount)),
            Self::MoveLeft(amount) => f.write_fmt(format_args!("MoveLeft({})", amount)),
            Self::Increment(amount) => f.write_fmt(format_args!("Increment({})", amount)),
            Self::Decrement(amount) => f.write_fmt(format_args!("Decrement({})", amount)),
            Self::Output => f.write_str("Output"),
            Self::Input => f.write_str("Input"),
            Self::Loop(instructions) => f.write_fmt(format_args!("Loop({:#?})", instructions)),
            Self::MoveRightUntilZero(amount) => {
                f.write_fmt(format_args!("MoveRightUntilZero({})", amount))
            }
            Self::MoveLeftUntilZero(amount) => {
                f.write_fmt(format_args!("MoveLeftUntilZero({})", amount))
            }
            Self::SetToZero => f.write_str("SetToZero"),
        }
    }
}
