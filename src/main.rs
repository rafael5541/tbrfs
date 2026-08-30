use std::env;
use std::fs;
use std::io::{self, Read, Write};

#[derive(Debug, PartialEq)]
enum Char {
    PtrR,
    PtrL,
    MemI,
    MemD,
    OutChr,
    InChr,
    JmpP,
    JmpB,
}
#[derive(Debug, PartialEq)]
struct JumpTable {
    jump_pos: usize,
    jump_to: Option<usize>,
}
#[derive(Debug, Default)]
struct Program {
    instructions: Vec<Char>,
    jump_table: Vec<JumpTable>,
}
struct VM {
    tape: Vec<u8>,
    pointer: usize,
    pc: usize,
}
fn getchar() -> Option<u8> {
    let mut buffer = [0; 1];
    match io::stdin().lock().read_exact(&mut buffer) {
        Ok(_) => Some(buffer[0]),
        Err(_) => None,
    }
}
fn parse_program(program: &String) -> Result<Program, String> {
    let parse: Vec<char> = program.chars().collect();
    let mut parsed: Vec<Char> = Vec::new();
    let mut jump: Vec<JumpTable> = Vec::new();
    let mut temp: Vec<usize> = Vec::new();
    for (index, &char) in parse.iter().filter(|&&c| !c.is_whitespace()).enumerate() {
        if char == '>' {
            parsed.push(Char::PtrR);
        } else if char == '<' {
            parsed.push(Char::PtrL);
        } else if char == '+' {
            parsed.push(Char::MemI);
        } else if char == '-' {
            parsed.push(Char::MemD);
        } else if char == '.' {
            parsed.push(Char::OutChr);
        } else if char == ',' {
            parsed.push(Char::InChr);
        } else if char == '[' {
            parsed.push(Char::JmpP);
            jump.push(JumpTable {
                jump_pos: index,
                jump_to: None,
            });
            temp.push(index);
        } else if char == ']' {
            parsed.push(Char::JmpB);
            if let Some(start_index) = temp.pop() {
                if let Some(rec) = jump.iter_mut().find(|j| j.jump_pos == start_index) {
                    rec.jump_to = Some(index);
                }
                jump.push(JumpTable {
                    jump_pos: index,
                    jump_to: Some(start_index),
                });
            } else {
                return Err(format!("found ] at position {} but no matching [", index));
            }
        }
    }
    if !temp.is_empty() {
        return Err(format!("found [ at position {} but no matching ]", temp[0]));
    }
    Ok(Program {
        instructions: parsed,
        jump_table: jump,
    })
}
fn run<W: Write>(writer: &mut W, program: &Program, tape_size: Option<usize>) {
    let tape = vec![0; tape_size.unwrap_or(30000)];
    let mut vm: VM = VM {
        tape: tape,
        pointer: 0,
        pc: 0,
    };
    loop {
        match program.instructions[vm.pc] {
            Char::PtrR => {
                vm.pc += 1;
                vm.pointer = (vm.pointer.wrapping_add(1)) % vm.tape.len()
            }
            Char::PtrL => {
                vm.pc += 1;
                vm.pointer = (vm.pointer.wrapping_sub(1)) % vm.tape.len()
            }
            Char::MemI => {
                vm.pc += 1;
                vm.tape[vm.pointer] = vm.tape[vm.pointer].wrapping_add(1)
            }
            Char::MemD => {
                vm.pc += 1;
                vm.tape[vm.pointer] = vm.tape[vm.pointer].wrapping_sub(1)
            }
            Char::OutChr => {
                vm.pc += 1;
                write!(writer, "{}", vm.tape[vm.pointer] as char).unwrap()
            }
            Char::InChr => {
                if let Some(c) = getchar() {
                    vm.tape[vm.pointer] = c;
                    vm.pc += 1
                }
            }
            Char::JmpP => {
                if vm.tape[vm.pointer] != 0 {
                    vm.pc += 1
                } else {
                    if let Some(jump) = program.jump_table.iter().find(|j| j.jump_pos == vm.pc) {
                        vm.pc = jump.jump_to.unwrap()
                    }
                }
            }
            Char::JmpB => {
                if vm.tape[vm.pointer] == 0 {
                    vm.pc += 1
                } else {
                    if let Some(jump) = program.jump_table.iter().find(|j| j.jump_to == Some(vm.pc))
                    {
                        vm.pc = jump.jump_pos
                    }
                }
            }
        }
        if vm.pc == program.instructions.len() {
            break;
        }
    }
}
fn main() -> Result<(), String> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let mut parsed_program = Program::default();
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} <file.bf>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];
    match fs::read_to_string(file_path) {
        Ok(contents) => match parse_program(&contents) {
            Ok(program) => parsed_program = program,
            Err(e) => return Err(format!("could not parse bf program: {}", e)),
        },
        Err(err) => eprintln!("error reading file '{}': {}", file_path, err),
    }
    run(&mut handle, &parsed_program, None);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helloworld() {
        let mut buffer = Vec::new();
        let hello_world = ">++++++++[-<+++++++++>]<.>>+>-[+]++>++>+++[>[->+++<<+++>]<<]>-----.>->
            +++..+++.>-.<<+[>[+>+]>>]<--------------.>>.+++.------.--------.>+.>+."
            .to_string();
        let parse = parse_program(&hello_world);
        run(&mut buffer, &parse.unwrap(), None);
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "Hello World!\n")
    }
    #[test]
    fn test_fib() {
        let mut buffer = Vec::new();
        let fib = ">+>+<<+++++++++[->>[->+>+<<]<[->+<]>>>[-<<<+>>>]<[-<+>]<<<]>>>>>>>>+++++++++++++++++++++++++++++++++++++++++++++++++++++.."
            .to_string();
        let parse = parse_program(&fib);
        run(&mut buffer, &parse.unwrap(), None);
        let output = String::from_utf8(buffer).unwrap();
        assert_eq!(output, "55")
    }
}
