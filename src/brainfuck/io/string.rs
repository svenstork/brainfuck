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
use std::str::Chars;

use super::base::Stdin;
use super::base::Stdout;

#[derive(Debug)]
pub struct StdinString<'a> {
    chars: Chars<'a>,
}

impl<'a> StdinString<'a> {
    pub fn new(value: &'a str) -> Self {
        StdinString {
            chars: value.chars(),
        }
    }
}

impl<'a> Stdin for StdinString<'a> {
    fn read(self: &mut Self) -> Result<Option<char>, io::Error> {
        Ok(self.chars.next())
    }
}

#[derive(Debug)]
pub struct StdoutString {
    output: Vec<char>,
}

impl StdoutString {
    pub fn new() -> Self {
        StdoutString { output: vec![] }
    }
}

impl Stdout for StdoutString {
    fn write(self: &mut Self, value: char) {
        self.output.push(value)
    }
}

impl ToString for StdoutString {
    fn to_string(self: &Self) -> String {
        self.output.iter().collect()
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_helloworld() {
        let value = "Hello World!";
        let mut stdin = StdinString::new(value);
        let mut result = vec![];
        while let Ok(Some(value)) = stdin.read() {
            result.push(value);
        }

        assert_eq!(result.len(), value.len());
        assert_eq!(value, result.iter().collect::<String>());
    }
}
