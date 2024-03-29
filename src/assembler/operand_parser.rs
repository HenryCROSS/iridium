use crate::assembler::register_parser::register;
use crate::assembler::Token;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{digit1, multispace0},
    combinator::{map, map_res, opt},
    sequence::{delimited, preceded, tuple},
    IResult,
};

use super::label_parsers::label_usage;

pub fn integer_operand(input: &str) -> IResult<&str, Token> {
    // delimited(
    //     multispace0,
    //     preceded(
    //         tag("#"),
    //         map_res(digit1, |digits: &str| {
    //             digits
    //                 .parse::<u8>()
    //                 .map(|value| Token::IntegerOperand { value: digits as i32 })
    //         }),
    //     ),
    //     multispace0,
    // )(input)

    delimited(
        multispace0,
        preceded(
            tag("#"),
            map_res(
                tuple((opt(tag("-")), digit1)),
                |(sign, digits): (Option<&str>, &str)| {
                    let mut tmp = String::new();
                    if sign.is_some() {
                        tmp.push('-');
                    }
                    tmp.push_str(digits);
                    tmp.parse::<i32>().map(|value: i32| Token::IntegerOperand {value})
                },
            ),
        ),
        multispace0,
    )(input)
}

fn irstring(input: &str) -> IResult<&str, Token> {
    map(
        delimited(
            multispace0,
            delimited(tag("'"), take_until("'"), tag("'")),
            multispace0,
        ),
        |content: &str| Token::IrString {
            name: content.to_string(),
        },
    )(input)
}

pub fn operand(intput: &str) -> IResult<&str, Token> {
    alt((integer_operand, label_usage, register, irstring))(intput)
}

#[test]
fn test_parse_integer_operand() {
    use super::*;

    // Test a valid integer operand
    let result = integer_operand("#-10");
    assert_eq!(result.is_ok(), true);
    let (rest, value) = result.unwrap();
    assert_eq!(rest, "");
    assert_eq!(value, Token::IntegerOperand { value: -10 });

    // Test an invalid one (missing the #)
    let result = integer_operand("10");
    assert_eq!(result.is_ok(), false);
}

#[test]
fn test_parse_string_operand() {
    let result = irstring("'This is a test'");
    assert_eq!(result.is_ok(), true);
}
