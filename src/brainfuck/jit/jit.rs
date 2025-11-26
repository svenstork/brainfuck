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
use std::io::{self, Write};
use std::{
    io::{stdin, Read},
    mem,
    time::Instant,
};

use dynasmrt::ExecutableBuffer;

use super::super::{
    jit::ast::AST,
    vm::{program::Program, vm::EvalError},
};

extern "C" fn output(value: libc::c_char) {
    let mut stdout = io::stdout();
    let _ = stdout.write(&[value as u8]);
}

extern "C" fn input() -> libc::c_char {
    let mut one_byte = [0];
    stdin().read_exact(&mut one_byte).unwrap();
    match one_byte[0] as char {
        '\n' => {
            return 0x0 as libc::c_char;
        }
        c => {
            return c as libc::c_char;
        }
    }
}

// Function type of the generated assembly code
type JITFunction = unsafe extern "C" fn(
    extern "C" fn(libc::c_char) -> (),
    extern "C" fn() -> libc::c_char,
    *const libc::c_char,
) -> ();

#[derive(Debug)]
pub struct JIT {
    program_ast: AST,
    memory_size: usize,
}

impl JIT {
    pub fn new(code: &str, rle: bool, memory_size: usize) -> Result<Self, EvalError> {
        let now = Instant::now();
        let program = Program::new(code, rle)?;
        let program_ast = AST::new(program);
        let elapsed = now.elapsed();
        println!("Parsing time: {:.2?}", elapsed);

        Ok(JIT {
            program_ast,
            memory_size,
        })
    }

    pub fn disassemble(&self) -> String {
        self.program_ast.pretty_print()
    }

    pub fn generate_code(&self) -> ExecutableBuffer {
        self.program_ast.generate_code()
    }

    pub fn run(&self) -> Vec<u8> {
        let assembly_code = self.generate_code();

        let callback: JITFunction = unsafe { mem::transmute(assembly_code.as_ptr()) };
        let memory = vec![0; self.memory_size];

        unsafe { callback(output, input, memory.as_ptr() as *const libc::c_char) };
        memory
    }
}
