use std::{
    collections::{hash_map::Entry, HashMap},
    iter,
    num::Wrapping,
    vec::IntoIter,
};

use either::Either;

use crate::{instruction::Instruction, parser::Parser};

pub struct Optimizer<Iter>
where
    Iter: Iterator<Item = Instruction>,
{
    iter: Iter,
}

impl<'a> Optimizer<Parser<'a>> {
    pub fn new(parser: Parser<'a>) -> Self {
        Self {
            iter: parser,
        }
    }
}

impl Optimizer<IntoIter<Instruction>> {
    fn new(iter: IntoIter<Instruction>) -> Self {
        Self {
            iter,
        }
    }
}

impl<Iter> Optimizer<Iter>
where
    Iter: Iterator<Item = Instruction>,
{
    fn optimize_loop(&mut self, instructions: Vec<Instruction>) -> Instruction {
        if instructions.len() == 1 {
            match instructions[0] {
                Instruction::MoveRight { amount } => {
                    Instruction::MoveRightUntilZero { step_size: amount }
                }
                Instruction::MoveLeft { amount } => {
                    Instruction::MoveLeftUntilZero { step_size: amount }
                }
                Instruction::Increment { amount: 1 } | Instruction::Decrement { amount: 1 } => {
                    Instruction::SetToZero
                }
                _ => Instruction::Loop { instructions },
            }
        } else {
            self.unroll_loop(instructions)
        }
    }

    fn unroll_loop(&mut self, instructions: Vec<Instruction>) -> Instruction {
        let mut current_relative_cell = 0isize;
        let mut relative_cell_operations = HashMap::new();

        let instructions = Optimizer::<IntoIter<_>>::new(instructions.into_iter()).collect::<Vec<_>>();

        let unroll_possible = instructions.iter().all(|instruction| {
            match instruction {
                Instruction::MoveRight { amount } => current_relative_cell += *amount as isize,
                Instruction::MoveLeft { amount } => current_relative_cell -= *amount as isize,
                Instruction::Increment { amount } => {
                    match relative_cell_operations.entry(current_relative_cell) {
                        Entry::Occupied(entry) => {
                            let (increment, increment_amount) = entry.into_mut();
                            if *increment {
                                *increment_amount += *amount;
                            } else {
                                *increment_amount -= *amount;
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert((true, Wrapping(*amount)));
                        }
                    }
                }
                Instruction::Decrement { amount } => {
                    match relative_cell_operations.entry(current_relative_cell) {
                        Entry::Occupied(entry) => {
                            let (increment, increment_amount) = entry.into_mut();
                            if *increment {
                                *increment_amount -= *amount;
                            } else {
                                *increment_amount += *amount;
                            }
                        }
                        Entry::Vacant(entry) => {
                            entry.insert((false, Wrapping(*amount)));
                        }
                    }
                }
                _ => return false,
            }

            true
        });

        if unroll_possible && current_relative_cell == 0 {
            if let Some((false, Wrapping(1))) = relative_cell_operations.remove(&0) {
                if relative_cell_operations.is_empty() {
                    return Instruction::SetToZero;
                } else if relative_cell_operations.len() == 1 {
                    if let (relative_cell, (true, Wrapping(1))) = relative_cell_operations.iter().next().unwrap() {
                        if relative_cell > &0 {
                            return Instruction::MoveValueRight { amount: *relative_cell as usize };
                        } else {
                            return Instruction::MoveValueLeft { amount: relative_cell.unsigned_abs() };
                        }
                    }
                }

                let operation_count = relative_cell_operations.len();
                let instructions = relative_cell_operations.into_iter().enumerate().flat_map(
                    |(i, (relative_cell, (increment, Wrapping(amount))))| {
                        if amount == 0 {
                            Either::Left(iter::empty())
                        } else {
                            let movement = relative_cell - current_relative_cell;
                            current_relative_cell = relative_cell;

                            let movement_instruction = if movement > 0 {
                                Instruction::MoveRight {
                                    amount: movement as usize,
                                }
                            } else {
                                Instruction::MoveLeft {
                                    amount: movement.unsigned_abs(),
                                }
                            };

                            let increment_instruction = if increment {
                                Instruction::Increment { amount }
                            } else {
                                Instruction::Decrement { amount }
                            };

                            let additional_instructions = if i == operation_count - 1 {
                                let last_movement_instruction = if current_relative_cell > 0 {
                                    Instruction::MoveLeft {
                                        amount: current_relative_cell as usize,
                                    }
                                } else {
                                    Instruction::MoveRight {
                                        amount: current_relative_cell.unsigned_abs(),
                                    }
                                };

                                Either::Left(iter::once(last_movement_instruction))
                            } else {
                                Either::Right(iter::empty())
                            };

                            Either::Right(
                                [movement_instruction, increment_instruction]
                                    .into_iter()
                                    .chain(additional_instructions),
                            )
                        }
                    },
                ).collect();

                return Instruction::WithMultiplier { instructions };
            }
        }

        Instruction::Loop {
            instructions,
        }
    }
}

impl<Iter> Iterator for Optimizer<Iter>
where
    Iter: Iterator<Item = Instruction>,
{
    type Item = Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|instruction| match instruction {
            Instruction::Loop { instructions } => self.optimize_loop(instructions),
            _ => instruction,
        })
    }
}
