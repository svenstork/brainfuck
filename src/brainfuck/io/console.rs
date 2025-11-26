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
use std::io;
use std::io::stdin;
use std::io::stdout;
use std::io::Read;
use std::io::Write;

use super::base::Stdin;
use super::base::Stdout;

#[derive(Debug)]
pub struct StdinConsole {
    prompt: Option<String>,
}

impl StdinConsole {
    pub fn new(prompt: Option<String>) -> Self {
        Self { prompt }
    }
}

impl Stdin for StdinConsole {
    fn read(&mut self) -> Result<Option<char>, io::Error> {
        if let Some(p) = &self.prompt {
            print!("\n{}", p);
            io::stderr().flush()?;
        }
        let mut one_byte = [0];
        stdin().read_exact(&mut one_byte)?;
        match one_byte[0] as char {
            '\n' => {
                return Ok(Some(0x0 as char));
            }
            c => {
                return Ok(Some(c));
            }
        }
    }
}

#[derive(Debug)]
pub struct StdoutConsole {
    output: Vec<char>,
}

impl StdoutConsole {
    pub fn new() -> Self {
        Self { output: vec![] }
    }
}

impl Stdout for StdoutConsole {
    fn write(&mut self, value: char) {
        self.output.push(value);
        print!("{}", value);
        let _ = stdout().flush();
    }
}

impl ToString for StdoutConsole {
    fn to_string(&self) -> String {
        self.output.iter().collect()
    }
}
