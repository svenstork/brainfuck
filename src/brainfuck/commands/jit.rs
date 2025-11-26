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
use std::{fs::File, io::Write, path::PathBuf};

use super::super::{
    jit::jit::JIT,
    commands::{common::dump_memory, config::Config},
    vm::vm::EvalError,
};

pub fn jit_disassemble(config: Config) -> Result<(), EvalError> {
    let jit = JIT::new(&config.code, config.rle, config.memory_size)?;
    let root = jit.disassemble();
    print!("{}", root);
    Ok(())
}

pub fn jit_dump(config: Config, output: PathBuf) -> Result<(), EvalError> {
    let jit = JIT::new(&config.code, config.rle, config.memory_size)?;
    let code = jit.generate_code();
    let mut output_file = File::create(&output)?;
    output_file.write_all(&code)?;
    println!("Wrote {} bytes to {:?}", code.len(), output);
    println!("\nThe assembly code is for a function of the following signature:");
    println!("\nvoid run(\n\tvoid (*output)(char),\n\tchar (*input)(),\n\tchar *memory\n);");
    Ok(())
}

pub fn jit_run(config: Config) -> Result<(), EvalError> {
    let jit = JIT::new(&config.code, config.rle, config.memory_size)?;
    let memory = jit.run();
    dump_memory(config, &memory)?;
    Ok(())
}
