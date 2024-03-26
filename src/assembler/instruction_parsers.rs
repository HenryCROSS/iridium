use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parser::integer_operand;
use crate::assembler::register_parser::register;
use crate::assembler::Token;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{eof, map_res, value, map},
    sequence::{delimited, preceded, tuple},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub struct AssemblerInstruction {
    opcode: Token,
    label: Option<Token>,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

pub fn instruction_one(input: &str) -> IResult<&str, AssemblerInstruction> {
    map(
        tuple((
            opcode_load,
            register,
            integer_operand
        )),
        |(opcode, operand1, operand2)| AssemblerInstruction {
            label: None,
            opcode,
            operand1: Some(operand1),
            operand2: Some(operand2),
            operand3: None,
        }
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Opcode;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction_one("load $0 #100\n");
        assert_eq!(
            result,
            Ok((
                "",
                AssemblerInstruction {
                    label: None,
                    opcode: Token::Op { code: Opcode::LOAD },
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    operand3: None
                }
            ))
        );
    }
}
