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
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use log::info;

use super::brainfuck::commands::config::Config;

#[derive(Debug, Clone, Parser)]
#[command(version, about, long_about = None)]
pub struct CLIArgs {
    /// Increase verbosity. Can be provided multiple times to increase verbosity limit.
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Use Run Length Encoding (RLE).
    #[arg(short, long, default_value_t = false)]
    pub rle: bool,

    /// Set memory size of the interpreter
    #[arg(short, long, default_value_t=1<<12)]
    pub memory_size: usize,

    /// Dump memory after the execution
    #[arg(short, long)]
    pub dump_memory: Option<PathBuf>,

    /// Print execution time
    #[arg(short = 't', long, default_value_t = false)]
    pub print_timing: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Parser)]
pub enum Commands {
    /// Use interpreter for execution (slower)
    INT {
        #[command(subcommand)]
        command: IntCommand,
    },

    /// Use Just In Time Compiler (JIT) to execute program (faster)
    #[cfg(target_arch = "aarch64")]
    JIT {
        #[command(subcommand)]
        command: JITCommand,
    },
}

#[derive(Debug, Clone, Parser)]
pub enum IntCommand {
    /// Start the debugger
    Debug { filename: PathBuf },

    /// Run the program in the interpreter (slower)
    Run {
        /// Dump profiling data. This will create/override the 'profile.txt' file.
        #[clap(short, long)]
        profile: bool,

        /// Path to the file to execute
        filename: PathBuf,
    },

    /// Disassemble the program
    Disassemble {
        filename: PathBuf,

        /// The start index of the first instruction
        #[clap(short, long)]
        start: Option<usize>,

        /// The end index of the first instruction
        #[clap(short, long)]
        end: Option<usize>,
    },
}

#[derive(Debug, Clone, Parser)]
pub enum JITCommand {
    /// Dump the AST of the program.
    Ast { filename: PathBuf },

    /// Dump the generated assembly code into a file.
    Dump { filename: PathBuf, output: PathBuf },

    /// Run the program
    Run { filename: PathBuf },
}

impl CLIArgs {
    pub fn config(&self) -> Result<Config> {
        let memory_size = self.memory_size;
        let rle = self.rle;
        let memory_dump = self.dump_memory.clone();
        let filename = match &self.command {
            Commands::INT {
                command: IntCommand::Debug { filename },
            } => filename,
            Commands::INT {
                command:
                    IntCommand::Disassemble {
                        filename,
                        start: _,
                        end: _,
                    },
            } => filename,
            Commands::INT {
                command:
                    IntCommand::Run {
                        profile: _,
                        filename,
                    },
            } => filename,
            #[cfg(target_arch = "aarch64")]
            Commands::JIT {
                command: JITCommand::Run { filename },
            } => filename,
            #[cfg(target_arch = "aarch64")]
            Commands::JIT {
                command: JITCommand::Ast { filename },
            } => filename,
            #[cfg(target_arch = "aarch64")]
            Commands::JIT {
                command:
                    JITCommand::Dump {
                        filename,
                        output: _,
                    },
            } => filename,
        };
        info!("Debuging file: {}", filename.to_str().unwrap());
        let code = CLIArgs::read_file(filename)?;
        Ok(Config::new(code, memory_size, rle, memory_dump))
    }

    fn read_file(filename: &PathBuf) -> Result<String> {
        fs::read_to_string(filename)
            .context(format!("Cannot read file '{}'", filename.to_str().unwrap()))
    }
}
