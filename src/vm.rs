use crate::assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX};
use crate::instruction::Opcode;

pub struct VM {
    // it could know at compile time as list type
    pub registers: [i32; 32],
    // program counter
    pc: usize,
    pub program: Vec<u8>,
    heap: Vec<u8>,
    remainder: u32,
    // the result of the last comparison operation
    equal_flag: bool,
    ro_data: Vec<u8>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            heap: vec![],
            pc: 0,
            remainder: 0,
            equal_flag: false,
            ro_data: vec![],
        }
    }

    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        return opcode;
    }

    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        return result;
    }

    fn next_16_bits(&mut self) -> u16 {
        // read off 2 bytes from the stack, move the first byte up 8 bits
        let result = ((self.program[self.pc] as u16) << 8) | self.program[self.pc + 1] as u16;
        self.pc += 2;
        return result;
    }

    pub fn add_byte(&mut self, byte: u8) {
        self.program.push(byte);
    }

    pub fn add_bytes(&mut self, bytes: Vec<u8>) {
        for b in bytes {
            self.add_byte(b);
        }
    }

    /// Loops as long as instructions can be executed.
    pub fn run(&mut self) {
        // main exec loop, performance-critical
        let mut is_not_done = self.verify_header();

        if is_not_done {
            self.pc += 65;
        } else {
            println!("The header is not current");
            std::process::exit(1);
        }

        while is_not_done {
            is_not_done = self.execute_instruction();
        }
    }

    /// Executes one instruction. Meant to allow for more controlled execution of the VM
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn execute_instruction(&mut self) -> bool {
        if self.pc >= self.program.len() {
            return false;
        }

        match self.decode_opcode() {
            Opcode::HLT => {
                println!("HLT encountered");
                return false;
            }
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize; // We cast to usize so we can use it as an index into the array
                let number = self.next_16_bits() as u16;
                self.registers[register] = number as i32; // Our registers are i32s, so we need to cast it. We'll cover that later.
            }
            Opcode::ADD => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = reg1 + reg2;
            }
            Opcode::SUB => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = reg1 - reg2;
            }
            Opcode::MUL => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = reg1 * reg2;
            }
            Opcode::DIV => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = reg1 / reg2;
                self.remainder = (reg1 % reg2) as u32;
            }
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc += value as usize;
            }
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc -= value as usize;
            }
            // $EQ r0, r1, None
            Opcode::EQ => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];

                self.equal_flag = reg1 == reg2;
                self.next_8_bits();
            }
            Opcode::NEQ => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];

                self.equal_flag = reg1 != reg2;
                self.next_8_bits();
            }
            Opcode::GT => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];

                self.equal_flag = reg1 > reg2;
                self.next_8_bits();
            }
            Opcode::LT => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];

                self.equal_flag = reg1 < reg2;
                self.next_8_bits();
            }
            Opcode::GTE => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];

                self.equal_flag = reg1 >= reg2;
                self.next_8_bits();
            }
            Opcode::LTE => {
                let reg1 = self.registers[self.next_8_bits() as usize];
                let reg2 = self.registers[self.next_8_bits() as usize];

                self.equal_flag = reg1 <= reg2;
                self.next_8_bits();
            }
            Opcode::JEQ => {
                let reg = self.next_8_bits() as usize;
                let target = self.registers[reg];
                if self.equal_flag {
                    self.pc = target as usize;
                }
            }
            Opcode::NOP => {
                self.next_8_bits();
                self.next_8_bits();
                self.next_8_bits();
            }
            Opcode::ALOC => {
                let reg = self.next_8_bits() as usize;
                let bytes = self.registers[reg];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            }
            Opcode::INC => {
                let reg = self.next_8_bits() as usize;
                let target = self.registers[reg];
                self.registers[reg] = target + 1;
            }
            Opcode::DEC => {
                let reg = self.next_8_bits() as usize;
                let target = self.registers[reg];
                self.registers[reg] = target - 1;
            }
            Opcode::PRTS => {
                // PRTS takes one operand, either a starting index in the read-only section of the bytecode
                // or a symbol (in the form of @symbol_name), which will look up the offset in the symbol table.
                // This instruction then reads each byte and prints it, until it comes to a 0x00 byte, which indicates
                // termination of the string
                let starting_offset = self.next_16_bits() as usize;
                let mut ending_offset = starting_offset;
                let slice = self.ro_data.as_slice();
                // TODO: Find a better way to do this. Maybe we can store the byte length and not null terminate? Or some form of caching where we
                // go through the entire ro_data on VM startup and find every string and its ending byte location?
                while slice[ending_offset] != 0 {
                    ending_offset += 1;
                }
                let result = std::str::from_utf8(&slice[starting_offset..ending_offset]);
                match result {
                    Ok(s) => {
                        print!("{}", s);
                    }
                    Err(e) => {
                        println!("Error decoding string for prts instruction: {:#?}", e)
                    }
                };
            }
            _ => {
                println!("Unrecognized opcode found! Terminating!");
                return false;
            }
        }

        true
    }

    fn verify_header(&self) -> bool {
        if self.program[0..4] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_vm() -> VM {
        VM::new()
    }

    fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            prepension.push(byte.clone());
        }
        while prepension.len() <= PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut b);
        prepension
    }

    #[test]
    fn test_create_vm() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0)
    }

    #[test]
    fn test_opcode_hlt() {
        let mut test_vm = VM::new();
        let test_bytes = vec![5, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.pc, 66);
    }

    #[test]
    fn test_opcode_igl() {
        let mut test_vm = VM::new();
        let test_bytes = vec![200, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.pc, 66);
    }

    #[test]
    fn test_load_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![0, 0, 1, 244]; // Remember, this is how we represent 500 using two u8s in little endian format
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[0], 500);
    }

    #[test]
    fn test_add_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![0, 0, 1, 244, 0, 1, 1, 244, 1, 0, 1, 2]; // Remember, this is how we represent 500 using two u8s in little endian format
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 1000);
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 1;
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 2;
        test_vm.program = vec![7, 0, 0, 0, 6, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 10;
        test_vm.program = vec![9, 0, 1, 0, 9, 0, 1, 0];
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 20;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_jeq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 7;
        test_vm.equal_flag = true;
        test_vm.program = vec![15, 0, 0, 0, 17, 0, 0, 0, 17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.pc, 7);
    }

    #[test]
    fn test_aloc_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![17, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.heap.len(), 1024);
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![0, 0, 0, 2, 0, 1, 0, 25, 3, 0, 1, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.registers[2], 50);
    }
}
