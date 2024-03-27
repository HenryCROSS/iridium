use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parser::{integer_operand, operand};
use crate::assembler::register_parser::register;
use crate::assembler::Token;

use nom::character::complete::{alpha1, multispace0};
use nom::character::streaming::space0;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::digit1,
    combinator::{eof, map, map_res, opt, value},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use super::instruction_parsers::AssemblerInstruction;

fn directive_declaration(input: &str) -> IResult<&str, Token> {
    map(preceded(tag("."), alpha1), |name: &str| Token::Directive {
        name: name.to_string(),
    })(input)
}

fn directive_combined(input: &str) -> IResult<&str, AssemblerInstruction> {
    map(
        delimited(
            multispace0,
            tuple((
                preceded(tag("."), directive_declaration),
                opt(operand),
                opt(operand),
                opt(operand),
            )),
            multispace0,
        ),
        |(directive, o1, o2, o3)| AssemblerInstruction {
            opcode: None,
            directive: Some(directive),
            label: None,
            operand1: o1,
            operand2: o2,
            operand3: o3,
        },
    )(input)
}

pub fn directive(input: &str) -> IResult<&str, AssemblerInstruction> {
    alt((directive_combined,))(input)
}
