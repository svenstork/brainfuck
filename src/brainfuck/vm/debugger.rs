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
use std::cmp::min;
use std::collections::BTreeSet;

use super::super::io::console::{StdinConsole, StdoutConsole};
use super::super::vm::vm::EvalError;
use super::super::vm::vm::Normal;
use super::vm::VM;

#[derive(Debug)]
pub struct Debugger {
    vm: VM<Normal>,
    breakpoints: BTreeSet<usize>,
}

impl Debugger {
    pub fn new(code: &str, rle: bool, memory_size: usize) -> Result<Self, EvalError> {
        let stdin = StdinConsole::new(Some("INPUT: ".to_string()));
        let stdout = StdoutConsole::new();
        let vm = VM::<Normal>::new(code, Box::new(stdin), Box::new(stdout), rle, memory_size)?;
        let breakpoints = BTreeSet::new();
        Ok(Debugger { vm, breakpoints })
    }

    /// Returns the VM machine registers.
    ///
    /// # Returns
    ///     (IP, DATA_PTR)
    pub fn registers(&self) -> (usize, usize) {
        (self.vm.ip(), self.vm.data_ptr())
    }

    /// Add new breakpoint.
    ///
    /// # Parameters
    ///     value - index of the breakpoint
    ///
    pub fn breakpoints_add(&mut self, value: usize) {
        self.breakpoints.insert(value);
    }

    /// Remove a breakpoint .
    ///
    /// # Parameters
    ///     value - index of the breakpoint
    ///
    pub fn breakpoints_delete(&mut self, value: usize) {
        self.breakpoints.remove(&value);
    }

    /// Returns list of breakpoint indexes.
    ///
    /// # Returns
    ///     Vec<usize> of indexes.
    pub fn breakpoints_list(&self) -> Vec<usize> {
        self.breakpoints.iter().copied().collect()
    }

    pub fn program_list(&self, index: Option<usize>) -> String {
        let border = 3;
        let focus = index.unwrap_or(self.vm.ip());
        let start = focus.saturating_sub(border);
        let end = min(self.vm.program().code.len(), focus + border);
        let output = self.vm.program().listing(Some(start), Some(end));
        if let Ok(data) = output {
            return data.to_string();
        }
        String::new()
    }

    pub fn memory(&self, index: usize) -> Option<u8> {
        if index < self.vm.memory().len() {
            return Some(self.vm.memory()[index]);
        }
        None
    }

    /// Helper function to generate a hex dump of the given memory range
    pub fn memory_dump(&self, start: usize, len: usize) -> String {
        let element_count = 0x10;
        let mut aligned = start & (usize::MAX - (element_count - 1));
        let mut dump = String::new();

        while aligned < start + len {
            dump.push_str(&format!("{:04x} | ", aligned));

            // add hex bytes
            (0..element_count).for_each(|offset| {
                let address = aligned + offset;
                if address < start || start + len <= address {
                    dump.push_str("   ");
                } else if let Some(value) = self.memory(address) {
                    dump.push_str(&format!("{:02x} ", value));
                }
            });

            // Add separator
            dump.push_str("| ");

            // add hex bytes
            (0..element_count).for_each(|offset| {
                let address = aligned + offset;
                if address < start || start + len <= address {
                    dump.push(' ');
                } else if let Some(value) = self.memory(address) {
                    let c = value as char;
                    if c.is_ascii_alphanumeric() || c.is_ascii_punctuation() {
                        dump.push_str(&format!("{}", c));
                    } else {
                        dump.push('.');
                    }
                }
            });

            dump.push('\n');
            aligned += element_count;
        }

        dump
    }

    pub fn output(&self) -> String {
        self.vm.stdout().to_string()
    }

    pub fn step(&mut self) -> Result<bool, EvalError> {
        self.vm.execute_step()
    }

    pub fn run(&mut self) -> Result<bool, EvalError> {
        loop {
            if self.step()? {
                break;
            }
            // Check if we reached a break point
            let ip = self.vm.ip();
            if self.breakpoints.contains(&ip) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
