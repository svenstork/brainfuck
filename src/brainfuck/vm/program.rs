// BSD 3-Clause License
//
// Copyright (c) 2025, Sven Stork
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this
//    list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice,
//    this list of conditions and the following disclaimer in the documentation
//    and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived from
//    this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
use std::fmt::Display;
use std::{cmp::max, collections::HashMap};
use thiserror::Error;

use super::super::vm::vm::EvalError;
use super::opcode::OpCode;

#[derive(Debug)]
pub struct Program {
    pub code: Vec<OpCode>,
    pub jump_table: HashMap<usize, usize>,
}

#[derive(Debug, Error)]
pub enum ProgramError {
    #[error("Found unmatched bracket at: {0}")]
    MissingBracket(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Step {
    // The opcode of the current instruction
    pub opcode: OpCode,
    // The ip of the next instruction. In the case of a loop start/end this represent the true case
    pub then_ip: Option<usize>,
    // In the case of the loop start/end this represens the next ip in the case the condition is false
    pub else_ip: Option<usize>,
}

impl Step {
    fn new(opcode: OpCode, then_ip: Option<usize>, else_ip: Option<usize>) -> Self {
        Step {
            opcode,
            then_ip,
            else_ip,
        }
    }
}

impl Program {
    /// CReate a new program based on the provided brainfuck code.
    ///
    /// # Arguments
    /// - `program_string` (`&str`) - The brainfuck program code
    ///
    /// # Return
    ///
    /// # Errors
    ///
    pub fn new(program_string: &str, rle: bool) -> Result<Self, ProgramError> {
        let mut code: Vec<OpCode> = program_string
            .chars()
            .filter_map(OpCode::convert)
            .collect();

        if rle {
            code = code.iter().fold(vec![], |mut acc, current| -> Vec<OpCode> {
                // check if we can merge with the top
                if let Some(top) = acc.pop() {
                    match (top, current) {
                        (OpCode::DecDataPtr(count), OpCode::DecDataPtr(1)) => {
                            acc.push(OpCode::DecDataPtr(count + 1))
                        }
                        (OpCode::IncDataPtr(count), OpCode::IncDataPtr(1)) => {
                            acc.push(OpCode::IncDataPtr(count + 1))
                        }
                        (OpCode::DecValue(count), OpCode::DecValue(1)) => {
                            acc.push(OpCode::DecValue(count + 1))
                        }
                        (OpCode::IncValue(count), OpCode::IncValue(1)) => {
                            acc.push(OpCode::IncValue(count + 1))
                        }
                        (t, c) => {
                            acc.push(t);
                            acc.push(*c);
                        }
                    }
                } else {
                    acc.push(*current);
                }
                acc
            });
        }
        let mut jump_table = HashMap::new();

        // compute jump table
        let mut stack = vec![];
        let result_jump_table: Result<(), ProgramError> = code
            .iter()
            .enumerate().try_for_each(|(index, opcode)| match opcode {
                OpCode::LoopStart => {
                    stack.push(index);
                    Ok(())
                }
                OpCode::LoopEnd => {
                    // Check if we have unmatched closed bracket
                    if stack.is_empty() {
                        return Err(ProgramError::MissingBracket(index));
                    }
                    let start = stack.pop().unwrap();
                    jump_table.insert(start, index);
                    jump_table.insert(index, start);
                    Ok(())
                }
                _ => Ok(()),
            });
        result_jump_table?;
        // Check if we have unmatch open bracket
        if let Some(index) = stack.pop() {
            return Err(ProgramError::MissingBracket(index));
        }
        Ok(Program { code, jump_table })
    }

    /// Return the step at the provided position of the program.
    ///
    /// # Arguments
    /// - `ip` (`usize`) - The instruction pointer of the opcpde
    ///
    /// # Returns
    ///
    pub fn get_step(&self, ip: usize) -> Option<Step> {
        let opcode: OpCode = *self.code.get(ip)?;
        let next_ip = if ip + 1 < self.code.len() {
            Some(ip + 1)
        } else {
            None
        };
        Some(match opcode {
            // All other instructions are simply moving to the next instruction in the code
            OpCode::LoopStart => {
                let loop_end_index = self.jump_table.get(&ip)?;
                Step::new(
                    opcode,
                    next_ip,
                    if loop_end_index + 1 < self.code.len() {
                        Some(loop_end_index + 1)
                    } else {
                        None
                    },
                )
            }
            OpCode::LoopEnd => {
                let loop_start_index = self.jump_table.get(&ip)?;
                Step::new(opcode, Some(loop_start_index + 1), next_ip)
            }
            // All other instruction just move on to the next instruction
            _ => Step {
                opcode,
                then_ip: next_ip,
                else_ip: None,
            },
        })
    }

    pub fn listing(
        &self,
        start: Option<usize>,
        end: Option<usize>,
    ) -> Result<String, EvalError> {
        // compute jump table
        let mut stack = vec![];
        let mut max_indent = 0;
        let mut gutter_size = 0;
        let mut indents = vec![];
        self.code
            .iter()
            .enumerate()
            .for_each(|(index, opcode)| match opcode {
                OpCode::LoopStart => {
                    stack.push(index);
                    max_indent += 1;
                    gutter_size = max(gutter_size, max_indent);
                    indents.push(max_indent);
                }
                OpCode::LoopEnd => {
                    stack.pop().unwrap();
                    indents.push(max_indent);
                    max_indent -= 1;
                }
                _ => {
                    indents.push(max_indent);
                }
            });
        // add space for arrow and visual appeal
        gutter_size += 2;
        let mut output = String::new();
        self.code.iter().enumerate().for_each(|(index, opcode)| {
            // Check range selection if specified
            if let Some(start_line) = start
                && index < start_line
            {
                return;
            }
            if let Some(end_line) = end
                && end_line < index
            {
                return;
            }
            // Print "line number"
            output.push_str(&format!("{:04} ", index));
            let indent = indents[index];
            // print pipes
            if self.jump_table.contains_key(&index) {
                for _ in 0..indent - 1 {
                    // print!("│");
                    output.push('│');
                }
                output.push_str(if *opcode == OpCode::LoopStart {
                    "╭─"
                } else {
                    "╰─"
                });
                for _ in (indent)..gutter_size - 2 {
                    // print!("─");
                    output.push('─');
                }
                // print!(">")
                output.push('>');
            } else {
                // print pipes
                for _ in 0..indent {
                    // print!("│");
                    output.push('│');
                }
                for _ in indent..gutter_size {
                    // print!(" ");
                    output.push(' ');
                }
            }
            output.push_str(format!(" {} ({:?})\n", opcode.to_char(), opcode).as_str());
        });
        Ok(output)
    }
}

impl Display for Program {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code_string: String = self.code.iter().map(OpCode::to_char).collect();
        formatter.write_str(&code_string)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn test_missmatched_loops_open() {
        let code = "[[[]]";
        let program = Program::new(code, false);
        assert!(program.is_err());
    }

    #[test]
    fn test_missmatched_loops_closed() {
        let code = "[[[]]]]";
        let program = Program::new(code, false);
        assert!(program.is_err());
    }

    #[test]
    fn test_parse_helloworld() {
        let code = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        let program_result = Program::new(code, false);
        assert!(program_result.is_ok());
        let program = program_result.unwrap();

        // Check parsing
        assert_eq!(code, program.to_string());

        // Check random
        for (ip, expected) in vec![
            (0, Step::new(OpCode::IncValue(1), Some(1), None)),
            (9, Step::new(OpCode::IncDataPtr(1), Some(10), None)),
            (30, Step::new(OpCode::DecDataPtr(1), Some(31), None)),
            (51, Step::new(OpCode::Output, Some(52), None)),
        ] {
            assert_eq!(program.get_step(ip), Some(expected))
        }

        // Check loops
        for (start, end) in vec![(8, 48), (14, 33), (43, 45)] {
            assert_eq!(
                program.get_step(start),
                Some(Step::new(OpCode::LoopStart, Some(start + 1), Some(end + 1)))
            );
            assert_eq!(
                program.get_step(end),
                Some(Step::new(OpCode::LoopEnd, Some(start + 1), Some(end + 1)))
            );
        }
    }

    #[test]
    fn test_optimization() {
        let code = "++++++++++[----------]";
        let program_result = Program::new(code, true);
        assert!(program_result.is_ok());
        let program = program_result.unwrap();
        assert_eq!(
            program.code,
            vec![
                OpCode::IncValue(10),
                OpCode::LoopStart,
                OpCode::DecValue(10),
                OpCode::LoopEnd
            ]
        )
    }
}
