use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{multispace0, multispace1},
    combinator::value,
    sequence::{delimited, preceded},
    IResult,
};

use super::Token;
use crate::instruction::Opcode;

pub fn opcode_load(input: &str) -> IResult<&str, Token> {
    delimited(
        multispace0,
        value(Token::Op { code: Opcode::LOAD }, tag("load")),
        multispace0,
    )(input)
}

mod tests {
    use super::*;

    #[test]
    fn test_opcode_load() {
        // First tests that the opcode is detected and parsed correctly
        let result = opcode_load("load");
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::LOAD });
        assert_eq!(rest, "");

        // Tests that an invalid opcode isn't recognized
        let result = opcode_load("aold");
        assert_eq!(result.is_ok(), false);
    }
}
