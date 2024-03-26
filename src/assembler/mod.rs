pub mod opcode;
pub mod opcode_parsers;
pub mod register_parser;
pub mod operand_parser;
pub mod instruction_parsers;
pub mod program_parsers;

use crate::instruction::Opcode;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: u8 },
}
