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
/// The AOT compiler generates a function that has the following parameters:
///
/// 1. a function pointer the output function that expectes a u8 as its only parameter
/// 2. a function pointer the input function that has not parameters and returns the U8 the user typed
/// 3. a pointer to the memory of the program
///
/// The AOT compiler uses the following local registers:
/// X19 - output function
/// X20 - input function
/// X21 - memory pointer
use super::code_generator::CodeGenerator;
use dynasm::dynasm;
use dynasmrt::{Assembler, aarch64::Aarch64Relocation};
use dynasmrt::{DynamicLabel, DynasmApi, DynasmLabelApi, ExecutableBuffer};

#[derive(Debug)]
pub struct ARM64CodeGenerator {
    assembler: Assembler<Aarch64Relocation>,
}

impl ARM64CodeGenerator {
    pub fn new() -> Self {
        Self {
            assembler: dynasmrt::aarch64::Assembler::new().unwrap(),
        }
    }
}

impl CodeGenerator for ARM64CodeGenerator {
    fn finalize(self: Box<Self>) -> ExecutableBuffer {
        self.assembler.finalize().unwrap()
    }

    fn function_prolog(&mut self) {
        dynasm!(self.assembler
            ; .arch aarch64
            ; stp x29, x30, [sp, -0x30]!
            ; stp x19, x20, [sp, 0x10]
            ; stp x21, x22, [sp, 0x20]
            ; mov x29, sp
            ; mov x19, x0
            ; mov x20, x1
            ; mov x21, x2
            ; mov x22, xzr
        );
    }

    fn function_epilog(&mut self) {
        dynasm!(self.assembler
            ; .arch aarch64
            ; ldp x19, x20, [sp, 0x10]
            ; ldp x21, x22, [sp, 0x20]
            ; ldp x29, x30, [sp], 0x30
            ; ret
        );
    }

    fn set(&mut self, value: u8) {
        let val = value as u32;
        if value == 0 {
            dynasm!(self.assembler
                ; .arch aarch64
                ;  strb wzr, [x21]
            );
        } else {
            dynasm!(self.assembler
                ; .arch aarch64
                ; mov w13, val
                ; strb w13, [x21]
            );
        }
    }

    fn add_to(&mut self, offset: i32) {
        dynasm!(self.assembler
            ; .arch aarch64
            ; ldrb w13, [x21]
            ; ldurb w14, [x21, offset]
            ; add w14, w14, w13
            ; sturb w14, [x21, offset]
            ; mov x13, #0
            ; strb w13, [x21]
        );
    }

    fn update_value(&mut self, update: i32) {
        if update < 0 {
            dynasm!(self.assembler
                ; .arch aarch64
                ; ldrb w13, [x21]
                ; sub w13, w13, -update as u32
                ; strb w13, [x21]
            );
        } else {
            dynasm!(self.assembler
                ; .arch aarch64
                ; ldrb w13, [x21]
                ; add w13, w13, update as u32
                ; strb w13, [x21]
            );
        };
    }

    fn update_memory_ptr(&mut self, update: i32) {
        if update < 0 {
            dynasm!(self.assembler
                ; .arch aarch64
                ;  sub x21, x21, -update as u32
            );
        } else {
            dynasm!(self.assembler
                ; .arch aarch64
                ;  add x21, x21, update as u32
            );
        };
    }

    fn output(&mut self) {
        dynasm!(self.assembler
            ; .arch aarch64
            ; mov x0, xzr
            ; ldrb w0, [x21]
            ; blr x19
        );
    }

    fn input(&mut self) {
        dynasm!(self.assembler
            ; .arch aarch64
            ;  blr x20
            ; strb w0, [x21]
        );
    }

    fn loop_start(&mut self) -> (DynamicLabel, DynamicLabel) {
        let loop_start = self.assembler.new_dynamic_label();
        let loop_end = self.assembler.new_dynamic_label();
        dynasm!(self.assembler
            ; .arch aarch64
            ; => loop_start
            ; ldrb w13, [x21]
            ; cbz w13, => loop_end
        );
        (loop_start, loop_end)
    }
    fn loop_end(&mut self, loop_start: DynamicLabel, loop_end: DynamicLabel) {
        dynasm!(self.assembler
            ; .arch aarch64
            ; b => loop_start
            ; => loop_end
        );
    }
}
