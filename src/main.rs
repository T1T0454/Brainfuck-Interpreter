use crate::Command::*;
use crate::NodeType::*;
use clap::{App, Arg, ArgMatches};
use std::{
    fs::File,
    io::{stdin, BufReader, Error, Read},
};

const MEMORY_SIZE: i32 = 30000;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum Command {
    Default,      // Nothing will happen
    IncDP,        // '>' -> Increment the data pointer (to point to the next cell to the right).
    DecDP,        // '<' -> Decrement the data pointer (to point to the next cell to the left).
    IncByte,      // '+' -> Increment (increase by one) the byte at the data pointer.
    DecByte,      // '-' -> Decrement (decrease by one) the byte at the data pointer.
    OutByte,      // '.' -> Output the byte at the data pointer.
    InByte, // ',' -> Accept one byte of input, storing its value in the byte at the data pointer.
    JumpForward, // '[' -> If the byte at the data pointer is zero, then instead of moving the instruction pointer forward to the next command, jump it forward to the command after the matching ] command.
    JumpBackward, // ']' -> If the byte at the data pointer is nonzero, then instead of moving the instruction pointer forward to the next command, jump it back to the command after the matching [ command.
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum NodeType {
    Program,
    Loop,
    Operator,
}

struct Interpreter {
    memory: Vec<u8>,
    pointer: i32,
}
struct Node {
    node_type: NodeType,
    instruction: Command,
    childrens: Vec<Node>,
}

/// Command line initialization
fn cli_init() -> ArgMatches {
    App::new("BrainF*ck Interpreter")
        .version("1.0")
        .author("Samuel G.")
        .about("Does awesome things")
        .arg(
            Arg::new("file")
                .about("sets the file to use")
                .takes_value(true)
                .short('f')
                .long("file")
                .required(true),
        )
        .get_matches()
}

/// Interpreter initialization
fn interpreter_init() -> Interpreter {
    Interpreter {
        memory: vec![0; MEMORY_SIZE as usize],
        pointer: 0,
    }
}

fn read_file_to_string(path: &str) -> Result<String, Error> {
    let file = File::open(path)?;
    let mut result_string = String::new();

    let mut buf_reader = BufReader::new(file);
    buf_reader.read_to_string(&mut result_string)?;

    Ok(result_string)
}

/// read file with buffer and transform chars to operators
fn lexical_analysis(commands: String) -> Result<Vec<Command>, String> {
    let mut result: Vec<Command> = Vec::new();
    commands.chars().for_each(|c| match c {
        '>' => result.push(IncDP),
        '<' => result.push(DecDP),
        '+' => result.push(IncByte),
        '-' => result.push(DecByte),
        '.' => result.push(OutByte),
        ',' => result.push(InByte),
        '[' => result.push(JumpForward),
        ']' => result.push(JumpBackward),
        _ => {}
    });
    Ok(result)
}

/// Generates abstract syntactic tree
fn create_ast(node: &mut Node, commands: &[Command], index: &mut usize) {
    while *index < commands.len() {
        // println!("index - while: {}", index);
        match commands.get(*index) {
            Some(cmd) => match cmd {
                JumpForward => {
                    let mut new_node = Node {
                        node_type: Loop,
                        instruction: JumpForward,
                        childrens: Vec::new(),
                    };
                    *index += 1;
                    create_ast(&mut new_node, commands, index);
                    node.childrens.push(new_node);
                }
                JumpBackward => {
                    return;
                }
                _ => {
                    node.childrens.push(Node {
                        node_type: Operator,
                        instruction: *cmd,
                        childrens: Vec::new(),
                    });
                }
            },
            None => return,
        }
        *index += 1;
    }
}

/// print tree just to make sure
// fn print_ast(program: &Node, depth: i32) {
//     if depth == 0 {
//         println!("\nPrinting ast\n");
//     }
//     program.childrens.iter().for_each(|node: &Node| {
//         println!(
//             "{}{:?}  ---  {:?}",
//             " ".repeat((depth * 5) as usize),
//             node.node_type,
//             node.instruction
//         );
//         if node.childrens.len() > 0 {
//             print_ast(node, depth + 1);
//         }
//     });
// }

/// provide syntactic analysis
fn syntax_analysis(commands: Vec<Command>) -> Result<Node, String> {
    let mut stack: Vec<Command> = Vec::new();
    let filtered = commands
        .iter()
        .filter(|cmd| -> bool { **cmd == JumpForward || **cmd == JumpBackward });

    for cmd in filtered {
        match *cmd {
            JumpForward => stack.push(JumpForward),
            _ => match stack.pop() {
                Some(_) => {}
                None => return Err("missing bracket".to_string()),
            },
        }
    }
    if !stack.is_empty() {
        return Err("missing bracket".to_string());
    }

    let mut program: Node = Node {
        node_type: Program,
        instruction: Default,
        childrens: Vec::new(),
    };

    let mut pos: usize = 0;
    create_ast(&mut program, &commands, &mut pos);

    Ok(program)
}

// Read one byte from user's input
fn read_input() -> u8 {
    let mut buffer = [0; 1];
    if stdin().read_exact(&mut buffer).is_ok() {
        return buffer[0];
    }
    0
}

// Change pointer or memory according on command and index
fn execute_instruction(interpreter: &mut Interpreter, cmd: &Command, index: usize) {
    match cmd {
        IncDP => interpreter.pointer += 1,
        DecDP => interpreter.pointer -= 1,
        IncByte => interpreter.memory[index] += 1,
        DecByte => interpreter.memory[index] -= 1,
        InByte => interpreter.memory[index] = read_input(),
        OutByte => print!("{}", interpreter.memory[index] as char),
        _ => {}
    };
}

fn run_program(interpreter: &mut Interpreter, ast: &Node) {
    ast.childrens.iter().for_each(|node| {
        // actual index in memory vector
        let mut index: usize =
            (((interpreter.pointer % MEMORY_SIZE) + MEMORY_SIZE) % MEMORY_SIZE) as usize;
        match node.node_type {
            Loop => {
                while interpreter.memory[index] != 0 {
                    run_program(interpreter, node);
                    index = (((interpreter.pointer % MEMORY_SIZE) + MEMORY_SIZE) % MEMORY_SIZE)
                        as usize;
                }
            }
            Operator => execute_instruction(interpreter, &node.instruction, index),
            _ => {}
        }
    });
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // println!("Hello BrainFuck!");

    let cli = cli_init();
    // commands from file
    match cli.value_of("file") {
        Some(f) => {
            let loaded_string: String = read_file_to_string(f)?;
            let commands: Vec<Command> = lexical_analysis(loaded_string)?;
            let program_ast = syntax_analysis(commands)?;
            let mut interpreter = interpreter_init();
            run_program(&mut interpreter, &program_ast);
        }
        None => panic!("Something went wrong"),
    }
    Ok(())
}
