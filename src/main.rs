use clap::Parser;

use std::path::Path;

use std::io::{prelude::*, BufReader};
use std::io;
use std::fs::File;

#[derive(Debug, Clone, Copy)]
enum Instruction {
    PointerIncrement(u8),
    PointerDecrement(u8),
    DataIncrement(u8),
    DataDecrement(u8),
    Input,
    Output,
    JumpForward(usize),
    JumpBackward(usize),
}

use Instruction::*;

impl Instruction {
    fn from_char(c: char) -> Option<Instruction> {
        match c {
            '>' => Some(PointerIncrement(1)),
            '<' => Some(PointerDecrement(1)),
            '-' => Some(DataDecrement(1)),
            '+' => Some(DataIncrement(1)),
            '.' => Some(Output),
            ',' => Some(Input),
            '[' => Some(JumpForward(0)),
            ']' => Some(JumpBackward(0)),
            _ => None,
        }
    }

    fn link_jumps(instructions: &mut Vec<Instruction>) {
        let mut forward_stack = vec![];
        let mut links = vec![];
        for (pc, instruction) in instructions.iter().enumerate() {
            match instruction {
                JumpForward(_) => forward_stack.push(pc),
                JumpBackward(_) => {
                    let last_forward = forward_stack.pop().expect("Unmatched []!");
                    links.push((last_forward, pc));
                },
                _ => (),
            }
        }
        if forward_stack.len() > 0 {
            panic!("Unmatched []!")
        }
        for (forward, backward) in links {
            instructions[forward] = JumpForward(backward-forward);
            instructions[backward] = JumpBackward(backward-forward);
            // println!("matching [] {forward}=>{backward}");
        }
    }

    fn compress_instructions(instructions: &Vec<Instruction>) -> Vec<Instruction> {
        let mut compressed_instructions: Vec<Instruction> = vec![instructions[0]];
        for i in 1..instructions.len() {
            let top = compressed_instructions.len() - 1;
            match (compressed_instructions[top], instructions[i]) {
                (PointerIncrement(n), PointerIncrement(1)) => 
                    compressed_instructions[top] = PointerIncrement(n.overflowing_add(1).0),
                (PointerDecrement(n), PointerDecrement(1)) => 
                    compressed_instructions[top] = PointerDecrement(n.overflowing_add(1).0),
                (DataIncrement(n), DataIncrement(1)) => 
                    compressed_instructions[top] = DataIncrement(n.overflowing_add(1).0),
                (DataDecrement(n), DataDecrement(1)) => 
                    compressed_instructions[top] = DataDecrement(n.overflowing_add(1).0),
                (_, instruction) => compressed_instructions.push(instruction),
            }
        }
        compressed_instructions
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
    let instructions: Vec<Instruction> = buf.lines().flat_map(|l| l.unwrap().chars().collect::<Vec<_>>())
        .map(Instruction::from_char)
        .filter_map(|f| f).collect();
    let mut instructions = Instruction::compress_instructions(&instructions);
    Instruction::link_jumps(&mut instructions);
    instructions
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
                self.forward.resize(forward_index+1,0);
            }
        } else {
            let rev_index: usize = (-index-1).try_into().unwrap();
            if rev_index >= self.forward.len() {
                self.backward.resize(rev_index+1,0);
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

    fn add(&mut self, index: isize, amount: u8) {
        self.allocate(index);
        if index >= 0 {
            let forward_index: usize = index as usize;
            self.forward[forward_index] = self.forward[forward_index].overflowing_add(amount).0;
        } else {
            let rev_index: usize = (-index-1).try_into().unwrap();
            self.backward[rev_index] = self.backward[rev_index].overflowing_add(amount).0;
        }
    }
    
    fn sub(&mut self, index: isize, amount: u8) {
        self.allocate(index);
        if index >= 0 {
            let forward_index: usize = index as usize;
            self.forward[forward_index] = self.forward[forward_index].overflowing_sub(amount).0;
        } else {
            let rev_index: usize = (-index-1).try_into().unwrap();
            self.backward[rev_index] = self.backward[rev_index].overflowing_sub(amount).0;
        }
    }
}

fn read_input() -> u8 {
    io::stdin().bytes().next().unwrap().unwrap()
}

fn main() {
    // Read in Program File
    let args = Args::parse();
    let path = Path::new(&args.path).canonicalize().unwrap();
    let program = instructions_from_file(path); 

    // Initialise Memory
    let mut memory = Memory::new();
    
    // Initialise program and stack "pointers"
    let mut pc: usize = 0;
    let mut sp: isize = 0;

    // Run Program
    loop {
        // print!("{pc}:{:?}\n", program[pc]);
        match program[pc] {
            PointerIncrement(n) => { sp += n as isize },
            PointerDecrement(n) => { sp -= n as isize },
            DataIncrement(n) => { memory.add(sp, n) },
            DataDecrement(n) => { memory.sub(sp, n) },
            Input => { 
                let input = read_input();
                memory.set(sp, input)
            },
            Output => { print!("{}",char::from(memory.get(sp))) },
            JumpForward(amount) => if memory.get(sp) == 0 { pc += amount },
            JumpBackward(amount) => if memory.get(sp) != 0 { pc -= amount },
        };
        pc += 1;
        if pc >= program.len() {
            break;
        }
    }
}
