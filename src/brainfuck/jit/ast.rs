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
use dynasmrt::ExecutableBuffer;
use log::debug;

use crate::brainfuck::jit::codegen::create_code_gen;

use super::super::jit::codegen::code_generator::CodeGenerator;
use super::super::vm::{opcode::OpCode, program::Program};

#[derive(Debug)]
pub enum AST {
    DecDataPtr(usize),
    DecValue(usize),
    IncDataPtr(usize),
    IncValue(usize),
    Input,
    Loop(Vec<AST>),
    Output,
    Program(Vec<AST>),
    Set(u8),
    AddTo(isize),
    // MultiplyOffset(isize, usize),
}

impl AST {
    pub fn new(program: Program) -> AST {
        AST::Program(AST::convert_opcodes(&program, 0, program.code.len())).optimize()
    }

    fn convert_opcodes(program: &Program, start: usize, end: usize) -> Vec<AST> {
        debug!("Convert opcodes from {} => {}", start, end);
        let mut nodes = vec![];
        let mut index = start;
        while index < end {
            let opcode = program.code[index];
            debug!("Converting {:?} to AST", opcode);
            let ast_node = match opcode {
                OpCode::DecDataPtr(count) => AST::DecDataPtr(count),
                OpCode::IncDataPtr(count) => AST::IncDataPtr(count),
                OpCode::DecValue(count) => AST::DecValue(count),
                OpCode::IncValue(count) => AST::IncValue(count),
                OpCode::Input => AST::Input,
                OpCode::Output => AST::Output,
                OpCode::LoopStart => {
                    let loop_end = program.jump_table.get(&index.clone()).unwrap();
                    debug!("Convert loopbody from {} => {}", index, loop_end);
                    let code_block = AST::convert_opcodes(program, index + 1, *loop_end);
                    index = *loop_end;
                    AST::Loop(code_block)
                }
                OpCode::LoopEnd => unreachable!(),
            };
            nodes.push(ast_node);
            index += 1;
        }
        nodes
    }

    pub fn optimize(self) -> Self {
        match self {
            AST::Program(codeblock) => {
                AST::Program(codeblock.into_iter().map(AST::optimize).collect())
            }
            AST::Loop(codeblock) => {
                let updated_codeblock: Vec<AST> =
                    codeblock.into_iter().map(|x| x.optimize()).collect();
                match updated_codeblock[..] {
                    // Set value to zero loops
                    [AST::DecValue(_)] | [AST::IncValue(_)] => AST::Set(0),
                    // Add move value e.g., ->>>+<<<
                    [
                        AST::DecValue(1),
                        AST::IncDataPtr(a),
                        AST::IncValue(1),
                        AST::DecDataPtr(b),
                    ] if a == b => AST::AddTo(b as isize),
                    // Add move value e.g., -<<<+>>>
                    [
                        AST::DecValue(1),
                        AST::DecDataPtr(b),
                        AST::IncValue(1),
                        AST::IncDataPtr(a),
                    ] if a == b => AST::AddTo(-(b as isize)),
                    // Add move value e.g., <<<<+>>>>-
                    [
                        AST::DecDataPtr(b),
                        AST::IncValue(1),
                        AST::IncDataPtr(a),
                        AST::DecValue(1),
                    ] if a == b => AST::AddTo(-(b as isize)),
                    // Add move value e.g.,>>>+<<<-
                    [
                        AST::IncDataPtr(a),
                        AST::IncValue(1),
                        AST::DecDataPtr(b),
                        AST::DecValue(1),
                    ] if a == b => AST::AddTo(b as isize),
                    // TODO: add multiply
                    // [
                    //     AST::DecValue(1),
                    //     AST::DecDataPtr(a),
                    //     AST::IncValue(v),
                    //     AST::IncDataPtr(b),
                    // ]
                    // | [
                    //     AST::DecValue(1),
                    //     AST::IncDataPtr(a),
                    //     AST::IncValue(v),
                    //     AST::DecDataPtr(b),
                    // ] if a == b => AST::MultiplyOffset(b as isize - a as isize, v),
                    // no optimizations
                    _ => AST::Loop(updated_codeblock),
                }
            }
            _ => self,
        }
    }

    pub fn pretty_print(&self) -> String {
        if let AST::Program(block) = self {
            return AST::pretty_print_with_indent(block, "".to_string());
        }
        "".to_string()
    }

    fn pretty_print_with_indent(nodes: &Vec<AST>, indent: String) -> String {
        nodes
            .iter()
            .map(|node| match node {
                AST::DecDataPtr(count) => format!("{}< ({})\n", indent, count),
                AST::IncDataPtr(count) => format!("{}> ({})\n", indent, count),
                AST::DecValue(count) => format!("{}- ({})\n", indent, count),
                AST::IncValue(count) => format!("{}+ ({})\n", indent, count),
                AST::Output => format!("{}.\n", indent),
                AST::Input => format!("{},\n", indent),
                AST::Loop(codeblock) => {
                    let sub_indent = format!("{}    ", indent);
                    let sub_string = AST::pretty_print_with_indent(codeblock, sub_indent);
                    format!("{}[\n{}{}]\n", indent, sub_string, indent)
                }
                AST::Set(value) => format!("{}set({})\n", indent, value),
                AST::AddTo(offset) => format!("{}addto({})\n", indent, offset),
                _ => unreachable!(),
            })
            .collect()
    }

    fn generate_code_block(nodes: &Vec<AST>, code_gen: &mut Box<dyn CodeGenerator>) {
        nodes
            .iter()
            .for_each(|node| node.generate_code_inner(code_gen))
    }

    fn generate_code_inner(&self, code_gen: &mut Box<dyn CodeGenerator>) {
        match self {
            AST::IncValue(count) => code_gen.update_value(*count as i32),
            AST::DecValue(count) => code_gen.update_value(-(*count as i32)),
            AST::IncDataPtr(count) => code_gen.update_memory_ptr(*count as i32),
            AST::DecDataPtr(count) => code_gen.update_memory_ptr(-(*count as i32)),
            AST::Output => code_gen.output(),
            AST::Input => code_gen.input(),
            AST::Set(value) => code_gen.set(*value),
            AST::AddTo(offset) => code_gen.add_to(*offset as i32),
            AST::Loop(codeblock) => {
                let (loop_start, loop_end) = code_gen.loop_start();
                AST::generate_code_block(codeblock, code_gen);
                code_gen.loop_end(loop_start, loop_end);
            }
            AST::Program(codeblock) => {
                code_gen.function_prolog();
                AST::generate_code_block(codeblock, code_gen);
                code_gen.function_epilog();
            }
        }
    }

    pub fn generate_code(&self) -> ExecutableBuffer {
        let mut code_generator = create_code_gen();
        self.generate_code_inner(&mut code_generator);
        code_generator.finalize()
    }
}
