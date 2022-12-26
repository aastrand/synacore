use byteorder::{LittleEndian, ReadBytesExt};
use std::env;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;

#[derive(Debug)]
struct VM {
    pub mem: Vec<u16>,
    pub regs: Vec<u16>,
    pub stack: Vec<u16>,
    pub ip: usize,
}

static LIMIT: u16 = 32768;

impl VM {
    pub fn new(mem: &Vec<u16>) -> VM {
        VM {
            mem: mem.clone(),
            regs: vec![0; 8],
            stack: vec![],
            ip: 0,
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
        println!("{:04x}: {}", self.ip, op);
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
                    self.print_op("halt");
                    break;
                }
                1 => {
                    let a = self.to_reg(self.mem[self.ip + 1]);
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    self.print_op(&format!("set  {} {:04x}", a, b));
                    self.regs[a as usize] = b;

                    self.ip += 3;
                }
                6 => {
                    let arg = self.convert_arg(self.mem[self.ip + 1]);
                    self.print_op(&format!("jmp  {:04x}", arg));

                    self.ip = arg as usize;
                }
                7 => {
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
                    let a = self.mem[self.ip + 1];
                    let b = self.convert_arg(self.mem[self.ip + 2]);
                    let c = self.convert_arg(self.mem[self.ip + 3]);
                    let r = (b + c) % LIMIT;
                    self.print_op(&format!("add  {} <= {} + {} = {}", self.to_reg(a), b, c, r));
                    self.store(a as usize, r);

                    self.ip += 4;
                }
                19 => {
                    print!(
                        "{}",
                        (self.convert_arg(self.mem[self.ip + 1])) as u8 as char
                    );

                    self.ip += 2;
                }
                21 => {
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
    }
}
