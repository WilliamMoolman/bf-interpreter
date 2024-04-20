use clap::Parser;
use text_io::read;

use std::path::Path;

use std::io::{prelude::*, BufReader};
use std::io;
use std::fs::File;

#[derive(Debug)]
enum Instruction {
    PointerIncrement,
    PointerDecrement,
    DataIncrement,
    DataDecrement,
    Input,
    Output,
    JumpForward,
    JumpBackward,
}

use Instruction::*;

impl Instruction {
    fn from_char(c: char) -> Option<Instruction> {
        match c {
            '>' => Some(PointerIncrement),
            '<' => Some(PointerDecrement),
            '-' => Some(DataDecrement),
            '+' => Some(DataIncrement),
            '.' => Some(Output),
            ',' => Some(Input),
            '[' => Some(JumpForward),
            ']' => Some(JumpBackward),
            _ => None,
        }
    }
} 


/// A CLI tool to interpret brainfuck source
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The root level folder to begin searching
    #[arg(default_value_t = String::from("."))]
    path: String,
}

fn instructions_from_file(filename: impl AsRef<Path>) -> Vec<Instruction> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines().flat_map(|l| l.unwrap().chars().collect::<Vec<_>>())
        .map(Instruction::from_char)
        .filter_map(|f| f).collect() 
}

struct Memory {
    forward: Vec<u8>,
    backward: Vec<u8>,
}

impl Memory {
    fn new() -> Memory {
        Memory { forward: vec![], backward: vec![] }
    }

    fn allocate(&mut self, index: isize) {
        if index >= 0 {
            let forward_index: usize = index as usize;
            if forward_index >= self.forward.len() {
                for _ in 0..=forward_index-self.forward.len() {
                    self.forward.push(0);
                }
            }
        } else {
            let rev_index: usize = (-index-1).try_into().unwrap();
            if rev_index >= self.forward.len() {
                for _ in 0..=rev_index-self.forward.len() {
                    self.backward.push(0);
                }
            }
        }
    }

    fn get(&mut self, index: isize) -> u8 {
        self.allocate(index);
        if index >= 0 {
            let forward_index: usize = index as usize;
            self.forward[forward_index]
        } else {
            let rev_index: usize = (-index-1).try_into().unwrap();
            self.forward[rev_index]
        }
    }

    fn set(&mut self, index: isize, value: u8) {
        self.allocate(index);
        if index >= 0 {
            let forward_index: usize = index as usize;
            self.forward[forward_index] = value;
        } else {
            let rev_index: usize = (-index-1).try_into().unwrap();
            self.forward[rev_index] = value;
        }
    }

    fn add(&mut self, index: isize, amount: i8) {
        self.allocate(index);
        if index >= 0 {
            let forward_index: usize = index as usize;
            self.forward[forward_index] = self.forward[forward_index].overflowing_add_signed(amount).0;
        } else {
            let rev_index: usize = (-index-1).try_into().unwrap();
            self.backward[rev_index] = self.backward[rev_index].overflowing_add_signed(amount).0;
        }
    }
}

fn read_input() -> u8 {
                // let input: String = read!(); 
                // let ascii: u8 = input.chars().next().unwrap() as u8;
    // let mut out = 0;
    // for i in io::stdin().bytes() {
    //     out = i.unwrap().clone();
    //     println!("read byte {}", out);
    // }
    // out
    io::stdin().bytes().next().unwrap().unwrap()
}

fn main() {
    // Read in Program File
    let args = Args::parse();
    let path = Path::new(&args.path).canonicalize().unwrap();

    let program = instructions_from_file(path); 
    // Initialise Memory
    
    let mut memory = Memory::new();

    let mut pc = 0;
    let mut sp = 0;

    loop {
        match program[pc] {
            PointerIncrement => { sp += 1 },
            PointerDecrement => { sp -= 1 },
            DataIncrement => { memory.add(sp, 1) },
            DataDecrement => { memory.add(sp, -1) },
            Input => { 
                let input = read_input();
                memory.set(sp, input)
            },
            Output => { print!("{}",char::from(memory.get(sp))) },
            JumpForward => {
                if memory.get(sp) == 0 {
                    let mut depth: isize = -1;
                    loop {
                        match program[pc] {
                            JumpForward => depth += 1,
                            JumpBackward => {
                                if depth ==0 { break; }
                                else {depth -= 1}
                            },
                            _ => (),
                        }
                        pc += 1;
                    }
                }

            },
            JumpBackward => { 
                if memory.get(sp) != 0 {
                    let mut depth: isize = -1;
                    loop {
                        match program[pc] {
                            JumpBackward => depth += 1,
                            JumpForward => {
                                if depth ==0 { break; }
                                else {depth -= 1}
                            },
                            _ => (),
                        }
                        pc -= 1;
                    }
                }


            },
        };
        pc += 1;
        if pc >= program.len() {
            break;
        }
    }
        

    // Loop until EOF
    //    Skip unwanted
    //    Handle valid characters
}
