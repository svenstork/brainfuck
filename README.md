# Sven's Brainfuck Interpreter

This is my simple brainfuck interpreter. This is a toy project to get my hands dirty with rust and learn the in and outs. Because this was a learning while doing project it might not be the best of code bases. The current implementation supports the foolwing features:

- interperter
    - debug: start the builtin debugger
    - disassemble: dump the code 
    - run: execute the sample program

- jit (only available on arm64)
    - ast: shows the internal AST tree
    - dump: Dump the generated assembly code.
    - run: execute the program using the jit compiler


# Extra Features

## Profiler
The interpreter has the option to collect profiling data and dump it into the `profile.txt` file. This can be handy when one needs to figure out hot spots in the code.

## Optimizations
Both the interpreter and the JIT version support the runtime length encoding (RLE) optimization. This is probably one of the best bang for your buck optimization for a brainfuck program.

The jit compile also supports a few more high level optimizations (e.g., replacing loops to reset a memory value to zero with a set to zero instruction).

# References
Here are some useful references that I have found useful while developing the interpreter.

- [ARM 64 Tutorial](https://mariokartwii.com/armv8/): Pretty good tutorial for arm assembly
- [Online ARM 64 assembler](https://ret.futo.org/arm64/): This was useful to get the binary encoding for the arm assembly codes. I used this before I switched to the dynasm crate.  

