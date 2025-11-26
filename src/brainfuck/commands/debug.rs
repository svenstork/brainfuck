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
use std::num::ParseIntError;
use std::process;

use super::super::commands::config::Config;
use super::super::vm::debugger::Debugger;
use super::super::vm::vm::EvalError;

use clap::{Parser, Subcommand};
use clap_repl::ClapEditor;
use clap_repl::reedline::FileBackedHistory;
use clap_repl::reedline::{DefaultPrompt, DefaultPromptSegment};

#[derive(Debug, Parser)]
#[command(name = "")] // This name will show up in clap's error messages, so it is important to set it to "".
enum DebuggerCommand {
    #[command(subcommand, visible_alias = "b")]
    /// Commands to manage breakpoints
    Breakpoint(BreakpointCommand),

    #[command(visible_alias = "c")]
    /// Run the code until the next break point is hit or the program ends
    Continue,

    #[command(visible_alias = "l")]
    /// List the instructions around the current instruction
    List {
        #[arg(value_parser = parse_usize_hex)]
        /// If specified show instruction around the specified index
        index: Option<usize>,
    },

    #[command(visible_alias = "m")]
    /// Inspect the memory
    Memory {
        #[arg(value_parser = parse_usize_hex)]
        /// The start address to display
        start: usize,

        #[arg(value_parser = parse_usize_hex)]
        /// The length of the data to dump
        len: usize,
    },

    #[command(visible_alias = "o")]
    /// Show the generated output
    Output,

    #[command(visible_alias = "r")]
    /// Show registers of the VM
    Registers,

    #[command(visible_alias = "s")]
    /// Execute the next instruction
    Step,

    #[command()]
    /// Ends the program
    Quit,
}

#[derive(Subcommand, Debug, Clone)]
pub enum BreakpointCommand {
    #[command(visible_alias = "l")]
    /// List all currently active breakpoints
    List,

    #[command(visible_alias = "c")]
    /// Create a new breakpoint at the specified line number
    Create {
        #[arg(value_parser = parse_usize_hex)]
        /// The line number to set the breakpoint on
        index: usize,
    },

    #[command(visible_alias = "d")]
    /// Delete a breakpoint by its ID
    Delete {
        #[arg(value_parser = parse_usize_hex)]
        /// The ID of the breakpoint to remove
        index: usize,
    },
}

pub fn debug(config: Config) -> Result<(), EvalError> {
    println!("Welcome to the brainfuck debugger. Use Ctrl+D to exit the debugger.");
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("brainfuck".to_owned()),
        ..DefaultPrompt::default()
    };
    let rl = ClapEditor::<DebuggerCommand>::builder()
        .with_prompt(Box::new(prompt))
        .with_editor_hook(|reed| {
            // Do custom things with `Reedline` instance here
            reed.with_history(Box::new(
                FileBackedHistory::with_file(10000, "~/.brainfuck.history".into()).unwrap(),
            ))
        })
        .build();

    let mut debugger = Debugger::new(&config.code, config.rle, config.memory_size)?;
    rl.repl(move |command| match command {
        DebuggerCommand::Breakpoint(cmd) => handle_breakpoint(&mut debugger, cmd),
        DebuggerCommand::Continue => handle_run(&mut debugger),
        DebuggerCommand::List { index } => handle_list(&mut debugger, index),
        DebuggerCommand::Memory { start, len } => handle_memory(&debugger, start, len),
        DebuggerCommand::Output => handle_output(&debugger),
        DebuggerCommand::Registers => handle_registers(&debugger),
        DebuggerCommand::Step => handle_step(&mut debugger),
        DebuggerCommand::Quit => process::exit(0),
    });
    Ok(())
}

fn handle_output(debugger: &Debugger) {
    println!("{}", debugger.output())
}

fn handle_registers(debugger: &Debugger) {
    let (ip, data_ptr) = debugger.registers();
    println!("ip   => 0x{:04x} ({})", ip, ip);
    println!("data => 0x{:04x} ({})", data_ptr, data_ptr);
}

fn handle_memory(debugger: &Debugger, start: usize, len: usize) {
    let dump = debugger.memory_dump(start, len);
    println!("{}", dump)
}

fn handle_list(debugger: &mut Debugger, index: Option<usize>) {
    let output = debugger.program_list(index);
    println!("{}", output)
}

fn handle_step(debugger: &mut Debugger) {
    match debugger.step() {
        Ok(true) => println!("Finished execution"),
        Ok(false) => {}
        Err(error) => {
            println!("{:?}", error)
        }
    }
}

fn handle_run(debugger: &mut Debugger) {
    match debugger.run() {
        Ok(true) => println!("Finished execution"),
        Ok(false) => {
            let (ip, _) = debugger.registers();
            println!("Run until breakpoint 0x{:04} ({})", ip, ip);
        }
        Err(error) => {
            println!("{:?}", error)
        }
    }
}

fn handle_breakpoint(debugger: &mut Debugger, command: BreakpointCommand) {
    match command {
        BreakpointCommand::Create { index } => {
            debugger.breakpoints_add(index);
        }
        BreakpointCommand::Delete { index } => {
            debugger.breakpoints_delete(index);
        }
        BreakpointCommand::List => {
            let breakpoints = debugger.breakpoints_list();
            if breakpoints.is_empty() {
                println!("No breakpoints defined");
                return;
            }
            println!("Found {} breakpoints", breakpoints.len());
            breakpoints
                .iter()
                .for_each(|index| println!("{:#08x} ({})", index, index));
        }
    }
}

/// Custom parser that accepts decimal or 0x-prefixed hexadecimal strings
fn parse_usize_hex(s: &str) -> Result<usize, ParseIntError> {
    if let Some(hex_str) = s.strip_prefix("0x") {
        // Parse as hexadecimal (base 16)
        usize::from_str_radix(hex_str, 16)
    } else {
        // Parse as standard decimal (base 10)
        s.parse::<usize>()
    }
}
