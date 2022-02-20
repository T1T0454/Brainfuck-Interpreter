# BrainFuck Interpreter

Interpreter of the simple programming language Brainfuck.
The project served as a school assignment at FI MUNI.
The project is divided into individual phases:

1. lexixal analysis
2. parsing
3. intermediate code generation
4. interpretation

Learn more about Brainfuck on [wikipedia](https://en.wikipedia.org/wiki/Brainfuck).

# Usage

```bash
#build the project
cargo build

#run via cargo
cargo run -- --file '<PATH_TO_FILE>'
```

# Example

```bash
cargo run -- --file test_files/print-0-to-99.txt
```
