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
use dynasmrt::{DynamicLabel, ExecutableBuffer};
use std::any::Any;

pub trait CodeGenerator: Any + 'static {
    fn finalize(self: Box<Self>) -> ExecutableBuffer;

    // Core operations necessary for brainfuck
    fn function_prolog(&mut self);
    fn function_epilog(&mut self);

    /// Call input function and write the read value into the current memory cell.
    fn input(&mut self);

    /// Wraps the provide {code} in a loop that is repeated until the current memory value is 0.
    fn loop_start(&mut self) -> (DynamicLabel, DynamicLabel);
    fn loop_end(&mut self, loop_start: DynamicLabel, loop_end: DynamicLabel);

    /// Call the output function with the current memory cell as parameter.
    fn output(&mut self);

    /// Update the memory pointer address by adding the {update} value.
    fn update_memory_ptr(&mut self, update: i32);

    /// Update the current memory value by adding the {update} value.
    fn update_value(&mut self, update: i32);

    // Enhanced operations. This are not composite operations

    /// Set the current memory value to the specified {value}.
    fn set(&mut self, value: u8);

    /// Takes the current memory cell value and adds it to the memory cell {offset} bytes away.
    /// After that the current memory cell will be set to 0.
    fn add_to(&mut self, offset: i32);
}
