use crate::assembler::Token;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::{eof, map_res, value},
    sequence::{delimited, preceded},
    IResult,
};

pub fn register(input: &str) -> IResult<&str, Token> {
    delimited(
        multispace0,
        preceded(
            tag("$"),
            map_res(digit1, |digits: &str| {
                digits
                    .parse::<u8>()
                    .map(|reg_num| Token::Register { reg_num })
            }),
        ),
        multispace0,
    )(input)
}

mod tests {
    use super::*;

    #[test]
    fn test_parse_register() {
        let result = register("$1");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::Register { reg_num: 1 });
        let result = register("0");
        assert_eq!(result.is_ok(), false);
        let result = register("$a");
        assert_eq!(result.is_ok(), false);
    }
}
