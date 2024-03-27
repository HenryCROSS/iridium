use crate::assembler::register_parser::register;
use crate::assembler::Token;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map_res,
    sequence::{delimited, preceded},
    IResult,
};

use super::label_parsers::label_usage;

pub fn integer_operand(input: &str) -> IResult<&str, Token> {
    delimited(
        multispace0,
        preceded(
            tag("#"),
            map_res(digit1, |digits: &str| {
                digits
                    .parse::<u8>()
                    .map(|value| Token::IntegerOperand { value })
            }),
        ),
        multispace0,
    )(input)
}

pub fn operand(intput: &str) -> IResult<&str, Token> {
    alt((integer_operand, register, label_usage))(intput)
}

#[test]
fn test_parse_integer_operand() {
    use super::*;

    // Test a valid integer operand
    let result = integer_operand("#10");
    assert_eq!(result.is_ok(), true);
    let (rest, value) = result.unwrap();
    assert_eq!(rest, "");
    assert_eq!(value, Token::IntegerOperand { value: 10 });

    // Test an invalid one (missing the #)
    let result = integer_operand("10");
    assert_eq!(result.is_ok(), false);
}
