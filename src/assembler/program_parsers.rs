use crate::assembler::instruction_parsers::{instruction, AssemblerInstruction};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::{eof, map, map_res, value},
    multi::many1,
    sequence::{delimited, preceded},
    IResult,
};

use super::SymbolTable;

#[derive(Debug, PartialEq, Clone)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>,
}

impl Program {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut program = vec![];
        for instruction in &self.instructions {
            program.append(&mut instruction.to_bytes(symbols));
        }

        program
    }
}

pub fn program(input: &str) -> IResult<&str, Program> {
    map(many1(instruction), |instructions| Program {
        instructions,
    })(input)
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_program() {

        let result = program("load $0 #100\nload $3 #120");
        assert_eq!(result.is_ok(), true);
        let (leftover, p) = result.unwrap();
        assert_eq!(leftover, "");
        assert_eq!(2, p.instructions.len());
        // TODO: Figure out an ergonomic way to test the AssemblerInstruction returned
    }

    #[test]
    fn test_program_to_bytes() {
        let result = program("load $0 #100\n");
        assert_eq!(result.is_ok(), true);
        let (_, program) = result.unwrap();
        let symbols = SymbolTable::new();
        let bytecode = program.to_bytes(&symbols);
        assert_eq!(bytecode.len(), 4);
        println!("{:?}", bytecode);
    }
}
