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
mod args;
mod brainfuck;

use anyhow::Result;
use args::{CLIArgs, Commands};
use clap::Parser;
use std::env;

#[cfg(target_arch = "aarch64")]
use brainfuck::commands::jit::{jit_disassemble, jit_dump, jit_run};
use brainfuck::commands::debug::debug;
use brainfuck::commands::disassemble::disassemble;
use brainfuck::commands::run::run;

#[cfg(target_arch = "aarch64")]
use args::JITCommand;
use args::IntCommand;

fn handle_cli_args(cli_args: CLIArgs) -> Result<()> {
    let config = cli_args.config()?;
    match cli_args.command {
        Commands::INT { command } => match command {
            IntCommand::Debug { filename: _ } => Ok(debug(config)?),
            IntCommand::Run {
                filename: _,
                profile,
            } => Ok(run(config, profile)?),
            IntCommand::Disassemble {
                filename: _,
                start,
                end,
            } => {
                print!("{}", disassemble(config, start, end)?);
                Ok(())
            }
        },
        #[cfg(target_arch = "aarch64")]
        Commands::JIT { command } => match command {
            JITCommand::Ast { filename: _ } => Ok(jit_disassemble(config)?),
            JITCommand::Dump {
                filename: _,
                output,
            } => Ok(jit_dump(config, output)?),
            JITCommand::Run { filename: _ } => Ok(jit_run(config)?),
        },
    }
}

/// Main function
fn main() -> Result<()> {
    env_logger::try_init()?;

    let cli = CLIArgs::parse();

    // Enable stack traces if we are very verbose
    if 1 < cli.verbose {
        // This is not the best way but seems the only way to do this programmatically
        unsafe {
            env::set_var("RUST_LIB_BACKTRACE", "1");
        }
    }
    handle_cli_args(cli)
}
