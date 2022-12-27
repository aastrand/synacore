use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs::File;
use std::io;
use std::io::Cursor;
use std::io::Read;
use std::io::{prelude::*, BufReader};
use std::path::Path;

#[macro_use]
extern crate text_io;

#[derive(Debug)]
struct VM {
    mem: Vec<u16>,
    symbols: HashMap<u16, String>,
    regs: Vec<u16>,
    stack: Vec<u16>,
    ip: usize,
    input_buffer: VecDeque<char>,
    debug: bool,
}

static LIMIT: u16 = 32768;

impl VM {
    pub fn new(mem: &Vec<u16>, symbols: &HashMap<u16, String>) -> VM {
        VM {
            mem: mem.clone(),
            symbols: symbols.clone(),
            regs: vec![0; 8],
            stack: vec![],
            ip: 0,
            input_buffer: VecDeque::new(),
            debug: false,
        }
    }

    fn reg_offset(&self, arg: u16) -> u16 {
        if arg >= LIMIT {
            arg - LIMIT
        } else {
            arg
        }
    }

    pub fn convert_arg(&self, arg: u16) -> u16 {
        if arg <= LIMIT - 1 {
            return arg;
        }
        if arg > 32767 && arg < 32776 {
            return self.regs[arg as usize - LIMIT as usize];
        } else {
            panic!("invalid addr: {}", arg)
        }
    }

    pub fn store(&mut self, addr: usize, val: u16) {
        if addr <= (LIMIT - 1).into() {
            self.mem[addr] = val;
        }
        if addr > 32767 && addr < 32776 {
            self.regs[addr - LIMIT as usize] = val;
        } else {
            panic!("invalid addr: {}", addr)
        }
    }

    fn print_op(&self, op: &str) {
        if self.debug {
            eprintln!(
            "{:04x}: {:<45} 0: {:04x} 1 {:04x} 2: {:04x} 3: {:04x} 4: {:04x} 5: {:04x} 6: {:04x} 7: {:04x} s({:>2}): {:04x}",
            self.ip, op, self.regs[0], self.regs[1], self.regs[2], self.regs[3], self.regs[4], self.regs[5], self.regs[6], self.regs[7], self.stack.len(), self.stack.last().unwrap_or(&0)
        );
        }
    }

    fn handle_debug(&mut self, line: &str) {
        let parts: &Vec<&str> = &line[1..].split(" ").collect();
        match parts[0] {
            "wmem" => {
                if parts.len() >= 3 {
                    let addr = u16::from_str_radix(parts[1], 16);
                    let val = u16::from_str_radix(parts[2], 16);

                    if addr.as_ref().is_ok() && val.as_ref().is_ok() {
                        let a = addr.unwrap();
                        let v = val.unwrap();
                        println!("DEBUG: wmem {:04x} {:04x}", a, v);
                        self.mem[a as usize] = v;
                    } else {
                        println!("DEBUG: error parsing arguments for wmem")
                    }
                } else {
                    println!("DEBUG: not enough arguments for wmem")
                }
            }
            _ => {}
        }
    }

    fn add_to_buffer(&mut self, input: &str) {
        for c in input.chars() {
            self.input_buffer.push_back(c);
        }
        self.input_buffer.push_back('\n');
    }

    pub fn run(&mut self) {
        self.add_to_buffer("take tablet");
        self.add_to_buffer("go doorway");
        self.add_to_buffer("go north");
        self.add_to_buffer("go north");
        self.add_to_buffer("go bridge");
        self.add_to_buffer("go continue");
        self.add_to_buffer("go down");
        self.add_to_buffer("go east");
        self.add_to_buffer("take empty lantern");
        self.add_to_buffer("go west");
        self.add_to_buffer("go west");
        self.add_to_buffer("go passage");
        self.add_to_buffer("go ladder");

        loop {
            if self.ip + 1 > self.mem.len() {
                println!("ran outside of memory range at ip={}", self.ip);
                break;
            }

            let instr = self.mem[self.ip];

            match instr {
                0 => {
                    // halt 0: stop execution and terminate the program
                    self.print_op("halt");
                    break;
                }
                1 => {
                    // set 1 a b: set register <a> to the value of <b>
                    let a = self.reg_offset(self.mem[self.ip + 1]);
                    let b = self.mem[self.ip + 2];
                    let b_val = self.convert_arg(self.mem[self.ip + 2]);
                    self.regs[a as usize] = b_val;

                    self.print_op(&format!(
                        "set  {} {:04x} ({:04x})",
                        a,
                        self.reg_offset(b),
                        b_val
                    ));
                    self.ip += 3;
                }
                2 => {
                    // push: 2 a: push <a> onto the stack
                    let a = self.mem[self.ip + 1];
                    let a_val = self.convert_arg(a);
                    self.stack.push(a_val);

                    self.print_op(&format!(
                        "push   {:04x} ({:04x})",
                        self.reg_offset(a),
                        a_val
                    ));
                    self.ip += 2;
                }
                3 => {
                    // pop: 3 a: remove the top element from the stack and write it into <a>; empty stack = error
                    let a = self.reg_offset(self.mem[self.ip + 1]);
                    let val = self.stack.pop().unwrap();
                    self.regs[a as usize] = val;

                    self.print_op(&format!("pop  {} ({:04x})", a, val));
                    self.ip += 2;
                }
                4 => {
                    // eq: 4 a b c: set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let c = self.mem[self.ip + 3];
                    let a_val = self.reg_offset(a);
                    let b_val = self.convert_arg(b);
                    let c_val = self.convert_arg(c);

                    if b_val == c_val {
                        self.regs[a_val as usize] = 1;
                    } else {
                        self.regs[a_val as usize] = 0;
                    }

                    self.print_op(&format!(
                        "eq   {} {:04x} ({:04x}) {:04x} ({:04x})",
                        a_val,
                        self.reg_offset(b),
                        b_val,
                        self.reg_offset(c),
                        c_val
                    ));
                    self.ip += 4;
                }
                5 => {
                    // gt: 5 a b c: set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let c = self.mem[self.ip + 3];
                    let a_val = self.reg_offset(a);
                    let b_val = self.convert_arg(b);
                    let c_val = self.convert_arg(c);

                    if b_val > c_val {
                        self.regs[a_val as usize] = 1;
                    } else {
                        self.regs[a_val as usize] = 0;
                    }

                    self.print_op(&format!(
                        "gt   {} {:04x} ({:04x}) {:04x} ({:04x})",
                        a_val,
                        self.reg_offset(b),
                        b_val,
                        self.reg_offset(c),
                        c_val
                    ));
                    self.ip += 4;
                }
                6 => {
                    // jmp: 6 a: jump to <a>
                    let a = self.mem[self.ip + 1];
                    let arg = self.convert_arg(a);

                    self.print_op(&format!("jmp    {:04x} ({:04x})", a, arg));
                    self.ip = arg as usize;
                }
                7 => {
                    // jt: 7 a b: if <a> is nonzero, jump to <b>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let a_val = self.convert_arg(a);
                    let b_val = self.convert_arg(b);

                    self.print_op(&format!(
                        "jt     {:04x} ({:04x}) {:04x} ({:04x})",
                        a, a_val, b, b_val
                    ));
                    if a_val != 0 {
                        self.ip = b_val as usize;
                    } else {
                        self.ip += 3;
                    }
                }
                8 => {
                    // jf: 8 a b: if <a> is zero, jump to <b>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let a_val = self.convert_arg(a);
                    let b_val = self.convert_arg(b);

                    self.print_op(&format!(
                        "jf     {:04x} ({:04x}) {:04x} ({:04x})",
                        a, a_val, b, b_val
                    ));
                    if a_val == 0 {
                        self.ip = b_val as usize;
                    } else {
                        self.ip += 3;
                    }
                }
                9 => {
                    // add: 9 a b c: assign into <a> the sum of <b> and <c> (modulo 32768)
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let c = self.mem[self.ip + 3];
                    let b_val = self.convert_arg(b);
                    let c_val = self.convert_arg(c);

                    let r = (b_val + c_val) % LIMIT;
                    self.store(a as usize, r);

                    self.print_op(&format!(
                        "add  {} {:04x} ({:04x}) {:04x} ({:04x})",
                        self.reg_offset(a),
                        self.reg_offset(b),
                        b_val,
                        self.reg_offset(c),
                        c_val
                    ));
                    self.ip += 4;
                }
                10 => {
                    // mult: 10 a b c: store into <a> the product of <b> and <c> (modulo 32768)
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let c = self.mem[self.ip + 3];
                    let b_val = self.convert_arg(b);
                    let c_val = self.convert_arg(c);

                    let r = ((b_val as u32 * c_val as u32) % LIMIT as u32) as u16;
                    self.store(a as usize, r);

                    self.print_op(&format!(
                        "mult {} {:04x} ({:04x}) {:04x} ({:04x})",
                        self.reg_offset(a),
                        self.reg_offset(b),
                        b_val,
                        self.reg_offset(c),
                        c_val
                    ));
                    self.ip += 4;
                }
                11 => {
                    // mod: 11 a b c: store into <a> the remainder of <b> divided by <c>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let c = self.mem[self.ip + 3];
                    let b_val = self.convert_arg(b);
                    let c_val = self.convert_arg(c);

                    let r = b_val % c_val;
                    self.store(a as usize, r);

                    self.print_op(&format!(
                        "mod  {} {:04x} ({:04x}) {:04x} ({:04x})",
                        self.reg_offset(a),
                        self.reg_offset(b),
                        b_val,
                        self.reg_offset(c),
                        c_val
                    ));
                    self.ip += 4;
                }
                12 => {
                    // and: 12 a b c: stores into <a> the bitwise and of <b> and <c>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let c = self.mem[self.ip + 3];
                    let b_val = self.convert_arg(b);
                    let c_val = self.convert_arg(c);

                    let r = b_val & c_val;
                    self.store(a as usize, r);

                    self.print_op(&format!(
                        "and  {} {:04x} ({:04x}) {:04x} ({:04x})",
                        self.reg_offset(a),
                        self.reg_offset(b),
                        b_val,
                        self.reg_offset(c),
                        c_val
                    ));
                    self.ip += 4;
                }
                13 => {
                    // or: 13 a b c: stores into <a> the bitwise or of <b> and <c>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let c = self.mem[self.ip + 3];
                    let b_val = self.convert_arg(b);
                    let c_val = self.convert_arg(c);

                    let r = b_val | c_val;
                    self.store(a as usize, r);

                    self.print_op(&format!(
                        "or   {} {:04x} ({:04x}) {:04x} ({:04x})",
                        self.reg_offset(a),
                        self.reg_offset(b),
                        b_val,
                        self.reg_offset(c),
                        c_val
                    ));
                    self.ip += 4;
                }
                14 => {
                    // not: 14 a b: stores 15-bit bitwise inverse of <b> in <a>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let b_val = self.convert_arg(b);

                    let r = !b_val & 0b0111_1111_1111_1111;
                    self.store(a as usize, r);

                    self.print_op(&format!(
                        "not  {} {:04x} ({:04x})",
                        self.reg_offset(a),
                        self.reg_offset(b),
                        b_val
                    ));
                    self.ip += 3;
                }
                15 => {
                    // rmem: 15 a b: read memory at address <b> and write it to <a>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let b_val = self.convert_arg(b);

                    let r = self.mem[b_val as usize];
                    self.store(a as usize, r);

                    self.print_op(&format!(
                        "rmem {} {:04x} ({:04x})",
                        self.reg_offset(a),
                        self.reg_offset(b),
                        b_val
                    ));
                    self.ip += 3;
                }
                16 => {
                    // wmem: 16 a b: write the value from <b> into memory at address <a>
                    let a = self.mem[self.ip + 1];
                    let b = self.mem[self.ip + 2];
                    let a_val = self.convert_arg(a);
                    let b_val = self.convert_arg(b);

                    self.mem[a_val as usize] = b_val;

                    self.print_op(&format!(
                        "wmem {} ({:04x}) {:04x} ({:04x})",
                        self.reg_offset(a),
                        a_val,
                        self.reg_offset(b),
                        b_val
                    ));
                    self.ip += 3;
                }
                17 => {
                    // call: 17 a: write the address of the next instruction to the stack and jump to <a>
                    let a = self.mem[self.ip + 1];
                    let a_val = self.convert_arg(a);
                    self.stack.push((self.ip + 2) as u16);

                    let symbol = self.symbols.get(&a_val);

                    if symbol.is_none() {
                        self.print_op(&format!("call {:04x} ({:04x})", self.reg_offset(a), a_val));
                    } else {
                        self.print_op(&format!(
                            "call {} {:04x} ({:04x})",
                            symbol.unwrap(),
                            self.reg_offset(a),
                            a_val
                        ));
                    }

                    if self.debug {
                        eprintln!("");
                        if let Some(sym) = symbol {
                            eprintln!("{}:", sym);
                        }
                    }
                    self.ip = a_val as usize;
                }
                18 => {
                    // ret: 18: remove the top element from the stack and jump to it; empty stack = halt
                    if self.stack.len() == 0 {
                        break;
                    }
                    let val = self.stack.pop().unwrap();

                    self.print_op(&format!("ret  {:04x}", val));
                    if self.debug {
                        eprintln!("");
                    }
                    self.ip = val as usize;
                }
                19 => {
                    // out: 19 a: write the character represented by ascii code <a> to the terminal
                    let a = self.mem[self.ip + 1];
                    let a_val = self.convert_arg(a);
                    let val = a_val as u8 as char;
                    print!("{}", val);

                    let mut debug_val: &str = &val.to_string();
                    if val == '\n' {
                        debug_val = "\\n";
                    }
                    self.print_op(&format!(
                        "out    {:04x} ({})",
                        self.reg_offset(a),
                        debug_val
                    ));
                    self.ip += 2;
                }
                20 => {
                    // in: 20 a: read a character from the terminal and write its ascii code to <a>;
                    // it can be assumed that once input starts, it will continue until a newline
                    // is encountered;
                    // this means that you can safely read whole lines from the keyboard
                    // and trust that they will be fully read
                    if self.input_buffer.len() == 0 {
                        let input: String = read!("{}\n");
                        for c in input.chars() {
                            self.input_buffer.push_back(c);
                        }
                        self.input_buffer.push_back('\n');
                        self.debug = true;

                        if self.input_buffer[0] == '.' {
                            self.handle_debug(&input)
                        }
                    }

                    let a = self.mem[self.ip + 1];
                    let r = self.input_buffer.pop_front().unwrap() as u16;
                    self.store(a as usize, r);

                    self.ip += 2;
                }
                21 => {
                    // noop: 21: no operation
                    self.print_op("noop");
                    self.ip += 1;
                }
                _ => {
                    panic!("not sure what to do with instruction {}", instr);
                }
            }
        }
    }
}

fn read_input(filename: &str) -> Result<Vec<u16>, io::Error> {
    let f = File::open(filename)?;
    let mut reader = BufReader::new(f);
    let mut buffer: Vec<u8> = Vec::new();
    reader.read_to_end(&mut buffer)?;
    let len = buffer.len();

    let mut mem: Vec<u16> = vec![];
    let mut rdr = Cursor::new(buffer);
    while (rdr.position() as usize) < len {
        let val = rdr.read_u16::<LittleEndian>().unwrap();
        mem.push(val);
    }

    Ok(mem)
}

fn read_symbols(filename: &str) -> HashMap<u16, String> {
    let file = File::open(Path::new(filename)).expect("No such file");
    let buf = BufReader::new(file);
    let mut table = HashMap::new();
    let lines: Vec<String> = buf
        .lines()
        .map(|l| l.expect("Could not parse line"))
        .collect();

    for line in lines {
        let parts = line
            .split(",")
            .map(|p| p.to_string())
            .collect::<Vec<String>>();
        table.insert(
            u16::from_str_radix(&parts[0], 16).unwrap(),
            parts[1].to_string(),
        );
    }

    table
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: synacore <file-to-execute>");
    }

    let mem = read_input(&args[1]).unwrap();
    let table = if args.len() > 2 {
        read_symbols(&args[2])
    } else {
        HashMap::new()
    };
    println!("{:?}", table);

    let mut vm = VM::new(&mem, &table);
    vm.run();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let program = vec![9, 32768, 32769, 4, 19, 32768];
        let mut vm = VM::new(&program, &HashMap::new());
        vm.run();

        assert_eq!(vm.regs[0], 4);
        assert_eq!(vm.ip, 6);
    }
}
