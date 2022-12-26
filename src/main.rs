use byteorder::{LittleEndian, ReadBytesExt};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;

#[macro_use]
extern crate text_io;

#[derive(Debug)]
struct VM {
    pub mem: Vec<u16>,
    pub regs: Vec<u16>,
    pub stack: Vec<u16>,
    pub ip: usize,
    pub input_buffer: VecDeque<char>,
}

static LIMIT: u16 = 32768;

impl VM {
    pub fn new(mem: &Vec<u16>) -> VM {
        VM {
            mem: mem.clone(),
            regs: vec![0; 8],
            stack: vec![],
            ip: 0,
            input_buffer: VecDeque::new(),
        }
    }

    fn to_reg(&self, arg: u16) -> u16 {
        arg - LIMIT
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
        /*println!(
            "{:04x}: {:<45} 0: {:04x} 1 {:04x} 2: {:04x} 3: {:04x} 4: {:04x} 5: {:04x} 6: {:04x} 7: {:04x} s({:>2}): {:04x}",
            self.ip, op, self.regs[0], self.regs[1], self.regs[2], self.regs[3], self.regs[4], self.regs[5], self.regs[6], self.regs[7], self.stack.len(), self.stack.last().unwrap_or(&0)
        );*/
    }

    pub fn run(&mut self) {
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
                    let a = self.to_reg(self.mem[self.ip + 1]);
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    self.regs[a as usize] = b;

                    self.print_op(&format!("set  {} {:04x}", a, b));
                    self.ip += 3;
                }
                2 => {
                    // push: 2 a: push <a> onto the stack
                    let a = self.convert_arg(self.mem[self.ip + 1]);
                    self.stack.push(a);

                    self.print_op(&format!("push {:04x}", a));
                    self.ip += 2;
                }
                3 => {
                    // pop: 3 a: remove the top element from the stack and write it into <a>; empty stack = error
                    let a = self.to_reg(self.mem[self.ip + 1]);
                    let val = self.stack.pop().unwrap();
                    self.regs[a as usize] = val;

                    self.print_op(&format!("pop  {:04x}", a));
                    self.ip += 2;
                }
                4 => {
                    // eq: 4 a b c: set <a> to 1 if <b> is equal to <c>; set it to 0 otherwise
                    let a = self.to_reg(self.mem[self.ip + 1]);
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);

                    if b == c {
                        self.regs[a as usize] = 1;
                    } else {
                        self.regs[a as usize] = 0;
                    }

                    self.print_op(&format!("eq   {} {:04x} {:04x}", a, b, c));
                    self.ip += 4;
                }
                5 => {
                    // gt: 5 a b c: set <a> to 1 if <b> is greater than <c>; set it to 0 otherwise
                    let a = self.to_reg(self.mem[self.ip + 1]);
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);

                    if b > c {
                        self.regs[a as usize] = 1;
                    } else {
                        self.regs[a as usize] = 0;
                    }

                    self.print_op(&format!("gt   {} {:04x} {:04x}", a, b, c));
                    self.ip += 4;
                }
                6 => {
                    // jmp: 6 a: jump to <a>
                    let arg = self.convert_arg(self.mem[self.ip + 1]);

                    self.print_op(&format!("jmp  {:04x}", arg));
                    self.ip = arg as usize;
                }
                7 => {
                    // jt: 7 a b: if <a> is nonzero, jump to <b>
                    let a = self.convert_arg(self.mem[self.ip + 1]);
                    let b = self.convert_arg(self.mem[self.ip + 2]);

                    self.print_op(&format!("jt   {} {:04x}", a, b));
                    if a != 0 {
                        self.ip = b as usize;
                    } else {
                        self.ip += 3;
                    }
                }
                8 => {
                    // jf: 8 a b: if <a> is zero, jump to <b>
                    let a = self.convert_arg(self.mem[self.ip + 1]);
                    let b = self.convert_arg(self.mem[self.ip + 2]);

                    self.print_op(&format!("jf   {} {:04x}", a, b));
                    if a == 0 {
                        self.ip = b as usize;
                    } else {
                        self.ip += 3;
                    }
                }
                9 => {
                    // add: 9 a b c: assign into <a> the sum of <b> and <c> (modulo 32768)
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);
                    let r = (b + c) % LIMIT;
                    self.store(a as usize, r);

                    self.print_op(&format!("add  {} {:04x} {:04x}", self.to_reg(a), b, c));
                    self.ip += 4;
                }
                10 => {
                    // mult: 10 a b c: store into <a> the product of <b> and <c> (modulo 32768)
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);
                    let r = ((b as u32 * c as u32) % LIMIT as u32) as u16;
                    self.store(a as usize, r);

                    self.print_op(&format!("mult {} {:04x} {:04x}", self.to_reg(a), b, c));
                    self.ip += 4;
                }
                11 => {
                    // mod: 11 a b c: store into <a> the remainder of <b> divided by <c>
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);
                    let r = b % c;
                    self.store(a as usize, r);

                    self.print_op(&format!("mod  {} {:04x} {:04x}", self.to_reg(a), b, c));
                    self.ip += 4;
                }
                12 => {
                    // and: 12 a b c: stores into <a> the bitwise and of <b> and <c>
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);
                    let r = b & c;
                    self.store(a as usize, r);

                    self.print_op(&format!("and  {} {:04x} {:04x}", self.to_reg(a), b, c));
                    self.ip += 4;
                }
                13 => {
                    // or: 13 a b c: stores into <a> the bitwise or of <b> and <c>
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);
                    let r = b | c;
                    self.store(a as usize, r);

                    self.print_op(&format!("or   {} {:04x} {:04x}", self.to_reg(a), b, c));
                    self.ip += 4;
                }
                14 => {
                    // not: 14 a b: stores 15-bit bitwise inverse of <b> in <a>
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    // 15 bit
                    let r = !b & 0b0111_1111_1111_1111;
                    self.store(a as usize, r);

                    self.print_op(&format!("not  {} {:04x}", self.to_reg(a), b));
                    self.ip += 3;
                }
                15 => {
                    // rmem: 15 a b: read memory at address <b> and write it to <a>
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let r = self.mem[b as usize];
                    self.store(a as usize, r);

                    self.print_op(&format!("rmem {} {:04x}", self.to_reg(a), b));
                    self.ip += 3;
                }
                16 => {
                    // wmem: 16 a b: write the value from <b> into memory at address <a>
                    let a = self.convert_arg(self.mem[self.ip + 1]);
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    self.mem[a as usize] = b;

                    self.print_op(&format!("wmem {:04x} {:04x}", a, b));
                    self.ip += 3;
                }
                17 => {
                    // call: 17 a: write the address of the next instruction to the stack and jump to <a>
                    let a = self.convert_arg(self.mem[self.ip + 1]);
                    self.stack.push((self.ip + 2) as u16);

                    self.print_op(&format!("call {:04x}", a));
                    self.ip = a as usize;
                }
                18 => {
                    // ret: 18: remove the top element from the stack and jump to it; empty stack = halt
                    if self.stack.len() == 0 {
                        break;
                    }
                    let val = self.stack.pop().unwrap();

                    self.print_op(&format!("ret  {:04x}", val));
                    self.ip = val as usize;
                }
                19 => {
                    // out: 19 a: write the character represented by ascii code <a> to the terminal
                    print!(
                        "{}",
                        (self.convert_arg(self.mem[self.ip + 1])) as u8 as char
                    );

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

fn main() -> io::Result<()> {
    let a: u16 = 0;
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: synacore <file-to-execute>");
    }

    let mem = read_input(&args[1]).unwrap();

    let mut vm = VM::new(&mem);
    vm.run();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let program = vec![9, 32768, 32769, 4, 19, 32768];
        let mut vm = VM::new(&program);
        vm.run();

        assert_eq!(vm.regs[0], 4);
        assert_eq!(vm.ip, 6);
    }
}
