use crate::assembler::instruction_parsers::{instruction_one, AssemblerInstruction};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::{eof, map, map_res, value},
    multi::many1,
    sequence::{delimited, preceded},
    IResult,
};

#[derive(Debug, PartialEq)]
pub struct Program {
    instructions: Vec<AssemblerInstruction>,
}

pub fn program(input: &str) -> IResult<&str, Program> {
    map(many1(instruction_one), |instructions| Program {
        instructions,
    })(input)
}

#[test]
fn test_parse_program() {
    use super::*;

    let result = program("load $0 #100\nload $3 #120");
    assert_eq!(result.is_ok(), true);
    let (leftover, p) = result.unwrap();
    assert_eq!(leftover, "");
    assert_eq!(2, p.instructions.len());
    // TODO: Figure out an ergonomic way to test the AssemblerInstruction returned
}
