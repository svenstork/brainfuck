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
use log::debug;
use std::io;
use thiserror::Error;

use super::super::io::base::Stdin;
use super::super::io::base::Stdout;

use super::opcode::OpCode;
use super::program::Program;
use super::program::ProgramError;

pub trait VMConfig {}

#[derive(Debug)]
pub struct Normal;

#[derive(Debug)]
pub struct Profiler {
    profile_data: Vec<usize>,
    ip_map: Vec<usize>,
}

impl VMConfig for Normal {}
impl VMConfig for Profiler {}

#[derive(Debug)]
pub struct VM<S: VMConfig> {
    ip: usize,
    data_ptr: usize,
    memory: Vec<u8>,
    program: Program,
    stdin: Box<dyn Stdin>,
    stdout: Box<dyn Stdout>,
    config: S,
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error("Program Error")]
    ProgramError(#[from] ProgramError),
    #[error("Memory Out of Bounds error")]
    MemoryOutOfBounds,
    #[error("Invalid Instruction Pointer")]
    InvalidInstructionPointer,
    #[error("I/O error")]
    IOError(#[from] io::Error),
}

impl<S: VMConfig> VM<S> {
    pub fn new(
        program_code: &str,
        stdin: Box<dyn Stdin>,
        stdout: Box<dyn Stdout>,
        rle: bool,
        memory_size: usize,
    ) -> Result<VM<Normal>, EvalError> {
        let program = Program::new(program_code, rle)?;

        Ok(VM {
            ip: 0,
            data_ptr: 0,
            memory: vec![0; memory_size],
            program,
            stdin,
            stdout,
            config: Normal {},
        })
    }

    /// Access the IP address of the VM
    pub fn ip(&self) -> usize {
        self.ip
    }

    pub fn data_ptr(&self) -> usize {
        self.data_ptr
    }

    pub fn stdout(&self) -> &Box<dyn Stdout> {
        &self.stdout
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn memory(&self) -> &Vec<u8> {
        &self.memory
    }

    /// Function to execute exactly one step if there is a step left.
    pub fn execute_step(&mut self) -> Result<bool, EvalError> {
        let step_option = self.program.get_step(self.ip);
        debug!(
            "Stepping ip={} => {:?}",
            self.ip,
            if step_option.is_some() {
                format!("{:?}", step_option.unwrap())
            } else {
                "NONE".to_string()
            }
        );
        if step_option.is_none() {
            return Ok(true);
        }
        let step = step_option.unwrap();
        let next_ip_option = match step.opcode {
            OpCode::DecDataPtr(count) => {
                if self.data_ptr < count {
                    return Err(EvalError::MemoryOutOfBounds);
                }
                self.data_ptr -= count;
                step.then_ip
            }
            OpCode::IncDataPtr(count) => {
                if self.data_ptr + count >= self.memory.len() {
                    return Err(EvalError::InvalidInstructionPointer);
                }
                self.data_ptr += count;
                step.then_ip
            }
            OpCode::DecValue(count) => {
                self.memory[self.data_ptr] = self.memory[self.data_ptr].wrapping_sub(count as u8);
                step.then_ip
            }
            OpCode::IncValue(count) => {
                self.memory[self.data_ptr] = self.memory[self.data_ptr].wrapping_add(count as u8);
                step.then_ip
            }
            OpCode::Input => {
                if let Some(value) = self.stdin.read()? {
                    self.memory[self.data_ptr] = value as u8;
                }
                step.then_ip
            }
            OpCode::Output => {
                self.stdout.write(self.memory[self.data_ptr] as char);
                step.then_ip
            }
            OpCode::LoopStart | OpCode::LoopEnd => {
                if self.memory[self.data_ptr] == 0 {
                    step.else_ip
                } else {
                    step.then_ip
                }
            }
        };
        if let Some(next_ip) = next_ip_option {
            self.ip = next_ip
        } else {
            return Ok(true);
        }
        Ok(false)
    }
}

impl VM<Normal> {
    pub fn run(&mut self) -> Result<(), EvalError> {
        loop {
            if self.execute_step()? {
                break;
            }
        }
        Ok(())
    }

    pub fn enable_profiler(self) -> VM<Profiler> {
        let ip_map = self.program().code.iter().fold(vec![0], |mut acc, opcode| {
            acc.push(acc[acc.len() - 1] + opcode.count());
            acc
        });
        let code_len = ip_map[ip_map.len() - 1];
        let profile_data = vec![0, code_len];

        VM {
            ip: self.ip,
            data_ptr: self.data_ptr,
            memory: self.memory,
            program: self.program,
            stdin: self.stdin,
            stdout: self.stdout,
            config: Profiler {
                profile_data,
                ip_map,
            },
        }
    }
}

impl VM<Profiler> {
    pub fn run(&mut self) -> Result<(), EvalError> {
        loop {
            let step_option = self.program.get_step(self.ip).unwrap();
            let count = step_option.opcode.count();
            let uncompressed_ip = self.config.ip_map[self.ip];
            (0..count).for_each(|index| self.config.profile_data[uncompressed_ip + index] += 1);
            if self.execute_step()? {
                break;
            }
        }
        Ok(())
    }

    pub fn profile_data(&self) -> &Vec<usize> {
        &self.config.profile_data
    }
}

#[cfg(test)]
mod test {
    use super::super::super::io::string::StdinString;
    use super::super::super::io::string::StdoutString;
    use super::*;
    use ctor::ctor;

    #[ctor]
    fn init_logging() {
        env_logger::try_init();
    }

    #[test]
    fn test_helloworld() -> Result<(), EvalError> {
        let hw = include_str!("../../../examples/helloworld.bf");
        let stdin = StdinString::new("");
        let stdout = StdoutString::new();
        let mut vm = VM::<Normal>::new(hw, Box::new(stdin), Box::new(stdout), false, 1 << 10)?;
        vm.run()?;
        assert_eq!("Hello World!\n", format!("{}", vm.stdout().to_string()));
        Ok(())
    }
}
