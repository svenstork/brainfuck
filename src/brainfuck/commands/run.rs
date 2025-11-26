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
use std::fs::File;
use std::io::Write;

use super::super::commands::config::Config;
use super::super::io::console::{StdinConsole, StdoutConsole};
use super::super::vm::vm::{EvalError, Normal, VM};

pub fn run(config: Config, profile: bool) -> Result<(), EvalError> {
    let stdin = StdinConsole::new(None);
    let stdout = StdoutConsole::new();
    let mut vm = VM::<Normal>::new(
        &config.code,
        Box::new(stdin),
        Box::new(stdout),
        config.rle,
        config.memory_size,
    )?;
    if profile {
        let mut vm_profile = vm.enable_profiler();
        let result = vm_profile.run();
        let profile_data = vm_profile
            .profile_data()
            .iter()
            .enumerate()
            .map(|(index, value)| format!("{:05}: {}\n", index, *value))
            .collect::<String>();
        File::create("profile.txt")?.write_all(profile_data.as_bytes())?;
        result
    } else {
        vm.run()
    }
}
