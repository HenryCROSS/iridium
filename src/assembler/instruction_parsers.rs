use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{eof, map, map_res, opt, value},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::assembler::directive_parsers::directive;
use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parser::{integer_operand, operand};
use crate::assembler::register_parser::register;
use crate::assembler::Token;

use super::{label_parsers::label_declaration, SymbolTable};

#[derive(Debug, PartialEq, Clone)]
pub struct AssemblerInstruction {
    pub opcode: Option<Token>,
    pub label: Option<Token>,
    pub directive: Option<Token>,
    pub operand1: Option<Token>,
    pub operand2: Option<Token>,
    pub operand3: Option<Token>,
}

impl AssemblerInstruction {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut results = vec![];
        match self.opcode {
            Some(Token::Op { code }) => match code {
                _ => {
                    results.push(code as u8);
                }
            },
            _ => {
                println!("Non-opcode found in opcode field");
                std::process::exit(1);
            }
        };

        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            match operand {
                Some(t) => AssemblerInstruction::extract_operand(t, &mut results, symbols),
                None => {}
            }
        }

        return results;
    }

    pub fn is_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn label_name(&self) -> Option<String> {
        if let Some(Token::LabelDeclaration { name }) = self.label.clone() {
            Some(name)
        } else {
            None
        }
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbols: &SymbolTable) {
        match t {
            Token::Register { reg_num } => {
                results.push(*reg_num);
            }
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }
            Token::LabelUsage { name } => {
                if let Some(value) = symbols.symbol_value(&name) {
                    let byte1 = value;
                    let byte2 = value >> 8;
                    results.push(byte2 as u8);
                    results.push(byte1 as u8);
                } else {
                    println!("No value found for {:?}", name);
                    std::process::exit(1);
                }
            }
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        }
    }
}

// no operand
fn instruction_zero(input: &str) -> IResult<&str, AssemblerInstruction> {
    map(opcode_load, |opcode| AssemblerInstruction {
        label: None,
        opcode: Some(opcode),
        directive: None,
        operand1: None,
        operand2: None,
        operand3: None,
    })(input)
}

// two operand
fn instruction_two(input: &str) -> IResult<&str, AssemblerInstruction> {
    map(
        tuple((opcode_load, register, integer_operand)),
        |(opcode, operand1, operand2)| AssemblerInstruction {
            label: None,
            opcode: Some(opcode),
            directive: None,
            operand1: Some(operand1),
            operand2: Some(operand2),
            operand3: None,
        },
    )(input)
}

// three operand
fn instruction_three(input: &str) -> IResult<&str, AssemblerInstruction> {
    map(
        tuple((opcode_load, register, register, register)),
        |(opcode, operand1, operand2, operand3)| AssemblerInstruction {
            label: None,
            opcode: Some(opcode),
            directive: None,
            operand1: Some(operand1),
            operand2: Some(operand2),
            operand3: Some(operand3),
        },
    )(input)
}

fn instruction_combined(input: &str) -> IResult<&str, AssemblerInstruction> {
    map(
        tuple((
            opt(label_declaration),
            opcode_load,
            opt(operand),
            opt(operand),
            opt(operand),
        )),
        |(label, opcode, operand1, operand2, operand3)| AssemblerInstruction {
            label,
            opcode: Some(opcode),
            directive: None,
            operand1,
            operand2,
            operand3,
        },
    )(input)
}

pub fn instruction(input: &str) -> IResult<&str, AssemblerInstruction> {
    alt((instruction_combined, directive))(input)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Opcode;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction("load $0 #100\n");
        assert_eq!(
            result,
            Ok((
                "",
                AssemblerInstruction {
                    label: None,
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    directive: None,
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    operand3: None
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let result = instruction("hlt\n");
        assert_eq!(
            result,
            Ok((
                "",
                AssemblerInstruction {
                    label: None,
                    opcode: Some(Token::Op { code: Opcode::HLT }),
                    directive: None,
                    operand1: None,
                    operand2: None,
                    operand3: None
                }
            ))
        );
    }
}
