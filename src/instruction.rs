use std::fmt::{Debug, Formatter, Result as FmtResult};

#[derive(Clone, PartialEq, Eq)]
pub enum Instruction {
    MoveRight { amount: usize },
    MoveLeft { amount: usize },
    Increment { amount: u8 },
    Decrement { amount: u8 },
    Output,
    Input,
    Loop { instructions: Vec<Instruction> },
    MoveRightUntilZero { step_size: usize },
    MoveLeftUntilZero { step_size: usize },
    SetToZero,
    WithMultiplier { instructions: Vec<Instruction> },
    MoveValueRight { amount: usize },
    MoveValueLeft { amount: usize },
}

impl Debug for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::MoveRight { amount } => f.write_fmt(format_args!("MoveRight({})", amount)),
            Self::MoveLeft { amount } => f.write_fmt(format_args!("MoveLeft({})", amount)),
            Self::Increment { amount } => f.write_fmt(format_args!("Increment({})", amount)),
            Self::Decrement { amount } => f.write_fmt(format_args!("Decrement({})", amount)),
            Self::Output => f.write_str("Output"),
            Self::Input => f.write_str("Input"),
            Self::Loop { instructions } => f.write_fmt(format_args!("Loop({:#?})", instructions)),
            Self::MoveRightUntilZero { step_size } => {
                f.write_fmt(format_args!("MoveRightUntilZero({})", step_size))
            }
            Self::MoveLeftUntilZero { step_size } => {
                f.write_fmt(format_args!("MoveLeftUntilZero({})", step_size))
            }
            Self::SetToZero => f.write_str("SetToZero"),
            Self::WithMultiplier { instructions } => {
                f.write_fmt(format_args!("WithMultiplier({:#?})", instructions))
            }
            Self::MoveValueRight { amount } => {
                f.write_fmt(format_args!("MoveValueRight({})", amount))
            }
            Self::MoveValueLeft { amount } => {
                f.write_fmt(format_args!("MoveValueLeft({})", amount))
            }
        }
    }
}
