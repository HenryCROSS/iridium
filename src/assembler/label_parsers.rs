use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, digit1, multispace0, space0},
    combinator::{eof, map, map_res, opt, value},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use crate::assembler::directive_parsers::directive;
use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parser::{integer_operand, operand};
use crate::assembler::register_parser::register;
use crate::assembler::Token;

pub fn label_declaration(input: &str) -> IResult<&str, Token> {
    map(
        terminated(alphanumeric1, preceded(tag(":"), multispace0)),
        |name: &str| Token::LabelDeclaration {
            name: name.to_string(),
        },
    )(input)
}

pub fn label_usage(input: &str) -> IResult<&str, Token> {
    map(
        preceded(tag("@"), preceded(multispace0, alphanumeric1)),
        |name: &str| Token::LabelUsage {
            name: name.to_string(),
        },
    )(input)
}

mod tests {
    use super::*;
    #[test]
    fn test_parse_label_declaration() {
        let result = label_declaration("test:");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(
            token,
            Token::LabelDeclaration {
                name: "test".to_string()
            }
        );
        let result = label_declaration("test");
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_parse_label_usage() {
        let result = label_usage("@test");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(
            token,
            Token::LabelUsage {
                name: "test".to_string()
            }
        );
        let result = label_usage("test");
        assert_eq!(result.is_ok(), false);
    }
}
