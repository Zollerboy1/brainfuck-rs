use std::vec::IntoIter;

use crate::{instruction::Instruction, parser::Parser};

pub struct Optimizer<Iter>
where
    Iter: Iterator<Item = Instruction>,
{
    iter: Iter,
}

impl<'a> Optimizer<Parser<'a>> {
    pub fn new(parser: Parser<'a>) -> Self {
        Self { iter: parser }
    }
}

impl Optimizer<IntoIter<Instruction>> {
    fn new(iter: IntoIter<Instruction>) -> Self {
        Self { iter }
    }
}

impl<Iter> Iterator for Optimizer<Iter>
where
    Iter: Iterator<Item = Instruction>,
{
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|instruction| match instruction {
            Instruction::Loop(instructions) => {
                if instructions.len() == 1 {
                    match instructions[0] {
                        Instruction::MoveRight(amount) => Instruction::MoveRightUntilZero(amount),
                        Instruction::MoveLeft(amount) => Instruction::MoveLeftUntilZero(amount),
                        Instruction::Increment(1) | Instruction::Decrement(1) => {
                            Instruction::SetToZero
                        }
                        _ => Instruction::Loop(instructions),
                    }
                } else {
                    Instruction::Loop(
                        Optimizer::<IntoIter<_>>::new(instructions.into_iter()).collect(),
                    )
                }
            }
            _ => instruction,
        })
    }
}
