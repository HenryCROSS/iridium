use nom::{
    character::complete::{alpha1, multispace0},
    combinator::map,
    sequence::delimited,
    IResult,
};

use super::Token;
use crate::instruction::Opcode;

pub fn opcode_load(input: &str) -> IResult<&str, Token> {
    map(delimited(multispace0, alpha1, multispace0), |code: &str| {
        Token::Op {
            code: Opcode::from(code),
        }
    })(input)
}

mod tests {
    use super::*;

    #[test]
    fn test_opcode_load() {
        // First tests that the opcode is detected and parsed correctly
        let result = opcode_load("load");
        println!("{:?}", result);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::LOAD });
        assert_eq!(rest, "");

        // Tests that an invalid opcode isn't recognized
        let result = opcode_load("aold");
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::IGL });
        assert_eq!(rest, "");
    }
}
