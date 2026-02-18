use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

fn main() {
    let sample = ">+8.*";
    println!("regex parser: {:?}", parse_regex(sample));
    println!("custom parser: {:?}", parse_custom(sample));
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedFormatSpec {
    sign: Option<Sign>,
    width: Option<Count>,
    precision: Option<Precision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sign {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Argument {
    Position(usize),
    Identifier(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Count {
    Integer(usize),
    Parameter(Argument),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Precision {
    Count(Count),
    Asterisk,
}

#[derive(Debug, Error, PartialEq, Eq)]
enum ParseError {
    #[error("invalid format_spec: {input}")]
    InvalidFormatSpec { input: String },
    #[error("invalid integer in {field}: {raw}")]
    InvalidInteger { field: &'static str, raw: String },
}

fn parse_regex(input: &str) -> Result<ParsedFormatSpec, ParseError> {
    static FORMAT_SPEC_RE: Lazy<Regex> = Lazy::new(|| {
        Regex::new(
            r"(?x)
            ^
            (?: .? [<^>] )?
            (?P<sign>[+-])?
            \#?
            0?
            (?P<width>
                (?:(?:\d+|(?:\p{XID_Start}\p{XID_Continue}*|_\p{XID_Continue}+))\$|\d+)
            )?
            (?:\.
                (?P<precision>
                    \*
                    |
                    (?:(?:\d+|(?:\p{XID_Start}\p{XID_Continue}*|_\p{XID_Continue}+))\$|\d+)
                )
            )?
            (?:
                \?
                | x\?
                | X\?
                | (?:\p{XID_Start}\p{XID_Continue}*|_\p{XID_Continue}+)
            )?
            $
            ",
        )
        .expect("hardcoded format_spec regex must be valid")
    });

    let Some(captures) = FORMAT_SPEC_RE.captures(input) else {
        return Err(ParseError::InvalidFormatSpec {
            input: input.to_owned(),
        });
    };

    let sign = captures
        .name("sign")
        .and_then(|m| parse_sign_token(m.as_str()));

    let width = captures
        .name("width")
        .map(|m| parse_count(m.as_str(), "width"))
        .transpose()?;

    let precision = captures
        .name("precision")
        .map(|m| parse_precision(m.as_str()))
        .transpose()?;

    Ok(ParsedFormatSpec {
        sign,
        width,
        precision,
    })
}

fn parse_custom(input: &str) -> Result<ParsedFormatSpec, ParseError> {
    let mut parser = CustomParser::new(input);

    parser.parse_alignment();
    let sign = parser.parse_sign();
    parser.consume_if('#');
    parser.consume_if('0');

    let width = parser.parse_optional_width()?;
    let precision = parser.parse_optional_precision()?;

    parser.parse_type()?;

    if !parser.is_eof() {
        return Err(ParseError::InvalidFormatSpec {
            input: input.to_owned(),
        });
    }

    Ok(ParsedFormatSpec {
        sign,
        width,
        precision,
    })
}

fn parse_sign_token(token: &str) -> Option<Sign> {
    match token {
        "+" => Some(Sign::Plus),
        "-" => Some(Sign::Minus),
        _ => None,
    }
}

fn parse_precision(raw: &str) -> Result<Precision, ParseError> {
    if raw == "*" {
        return Ok(Precision::Asterisk);
    }

    Ok(Precision::Count(parse_count(raw, "precision")?))
}

fn parse_count(raw: &str, field: &'static str) -> Result<Count, ParseError> {
    if let Some(argument) = raw.strip_suffix('$') {
        return Ok(Count::Parameter(parse_argument(argument, field)?));
    }

    Ok(Count::Integer(parse_usize(raw, field)?))
}

fn parse_argument(raw: &str, field: &'static str) -> Result<Argument, ParseError> {
    if raw.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(Argument::Position(parse_usize(raw, field)?));
    }

    if is_valid_identifier(raw) {
        return Ok(Argument::Identifier(raw.to_owned()));
    }

    Err(ParseError::InvalidFormatSpec {
        input: raw.to_owned(),
    })
}

fn parse_usize(raw: &str, field: &'static str) -> Result<usize, ParseError> {
    raw.parse::<usize>()
        .map_err(|_| ParseError::InvalidInteger {
            field,
            raw: raw.to_owned(),
        })
}

fn is_align(ch: char) -> bool {
    matches!(ch, '<' | '^' | '>')
}

fn is_valid_identifier(identifier: &str) -> bool {
    let mut chars = identifier.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if first == '_' {
        let Some(next) = chars.next() else {
            return false;
        };

        if !unicode_ident::is_xid_continue(next) {
            return false;
        }

        return chars.all(unicode_ident::is_xid_continue);
    }

    if !unicode_ident::is_xid_start(first) {
        return false;
    }

    chars.all(unicode_ident::is_xid_continue)
}

struct CustomParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> CustomParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn parse_alignment(&mut self) {
        let Some(first) = self.peek_char() else {
            return;
        };

        if is_align(first) {
            self.bump_char();
            return;
        }

        let mut chars = self.rest().chars();
        let _ = chars.next();
        let Some(second) = chars.next() else {
            return;
        };

        if is_align(second) {
            self.bump_char();
            self.bump_char();
        }
    }

    fn parse_sign(&mut self) -> Option<Sign> {
        let sign = self.peek_char().and_then(|ch| match ch {
            '+' => Some(Sign::Plus),
            '-' => Some(Sign::Minus),
            _ => None,
        });

        if sign.is_some() {
            self.bump_char();
        }

        sign
    }

    fn parse_optional_width(&mut self) -> Result<Option<Count>, ParseError> {
        let Some(first) = self.peek_char() else {
            return Ok(None);
        };

        if first.is_ascii_digit() {
            let number = self.consume_digits();
            if self.consume_if('$') {
                return Ok(Some(Count::Parameter(Argument::Position(parse_usize(
                    &number, "width",
                )?))));
            }

            return Ok(Some(Count::Integer(parse_usize(&number, "width")?)));
        }

        if !is_identifier_start(first) {
            return Ok(None);
        }

        let checkpoint = self.index;
        let Some(identifier) = self.consume_identifier() else {
            return Ok(None);
        };

        if self.consume_if('$') {
            return Ok(Some(Count::Parameter(Argument::Identifier(identifier))));
        }

        self.index = checkpoint;
        Ok(None)
    }

    fn parse_optional_precision(&mut self) -> Result<Option<Precision>, ParseError> {
        if !self.consume_if('.') {
            return Ok(None);
        }

        if self.consume_if('*') {
            return Ok(Some(Precision::Asterisk));
        }

        let count = self.parse_required_count("precision")?;
        Ok(Some(Precision::Count(count)))
    }

    fn parse_required_count(&mut self, field: &'static str) -> Result<Count, ParseError> {
        let Some(first) = self.peek_char() else {
            return Err(ParseError::InvalidFormatSpec {
                input: self.input.to_owned(),
            });
        };

        if first.is_ascii_digit() {
            let number = self.consume_digits();
            if self.consume_if('$') {
                return Ok(Count::Parameter(Argument::Position(parse_usize(
                    &number, field,
                )?)));
            }

            return Ok(Count::Integer(parse_usize(&number, field)?));
        }

        if !is_identifier_start(first) {
            return Err(ParseError::InvalidFormatSpec {
                input: self.input.to_owned(),
            });
        }

        let Some(identifier) = self.consume_identifier() else {
            return Err(ParseError::InvalidFormatSpec {
                input: self.input.to_owned(),
            });
        };

        if !self.consume_if('$') {
            return Err(ParseError::InvalidFormatSpec {
                input: self.input.to_owned(),
            });
        }

        Ok(Count::Parameter(Argument::Identifier(identifier)))
    }

    fn parse_type(&mut self) -> Result<(), ParseError> {
        if self.is_eof() {
            return Ok(());
        }

        if self.consume_str("x?") || self.consume_str("X?") || self.consume_if('?') {
            return Ok(());
        }

        if self.consume_identifier().is_some() {
            return Ok(());
        }

        Err(ParseError::InvalidFormatSpec {
            input: self.input.to_owned(),
        })
    }

    fn consume_identifier(&mut self) -> Option<String> {
        let rest = self.rest();
        let mut chars = rest.char_indices();

        let (_, first) = chars.next()?;

        if first == '_' {
            let (next_offset, next) = chars.next()?;
            if !unicode_ident::is_xid_continue(next) {
                return None;
            }

            let mut end = next_offset + next.len_utf8();
            for (offset, ch) in chars {
                if !unicode_ident::is_xid_continue(ch) {
                    break;
                }
                end = offset + ch.len_utf8();
            }

            let identifier = &rest[..end];
            self.index += end;
            return Some(identifier.to_owned());
        }

        if !unicode_ident::is_xid_start(first) {
            return None;
        }

        let mut end = first.len_utf8();
        for (offset, ch) in chars {
            if !unicode_ident::is_xid_continue(ch) {
                break;
            }
            end = offset + ch.len_utf8();
        }

        let identifier = &rest[..end];
        self.index += end;
        Some(identifier.to_owned())
    }

    fn consume_digits(&mut self) -> String {
        let rest = self.rest();
        let mut end = 0;

        for (offset, ch) in rest.char_indices() {
            if !ch.is_ascii_digit() {
                break;
            }
            end = offset + ch.len_utf8();
        }

        let digits = &rest[..end];
        self.index += end;
        digits.to_owned()
    }

    fn consume_if(&mut self, expected: char) -> bool {
        let Some(ch) = self.peek_char() else {
            return false;
        };

        if ch != expected {
            return false;
        }

        self.bump_char();
        true
    }

    fn consume_str(&mut self, token: &str) -> bool {
        if !self.rest().starts_with(token) {
            return false;
        }

        self.index += token.len();
        true
    }

    fn bump_char(&mut self) {
        let Some(ch) = self.peek_char() else {
            return;
        };

        self.index += ch.len_utf8();
    }

    fn peek_char(&self) -> Option<char> {
        self.rest().chars().next()
    }

    fn rest(&self) -> &'a str {
        &self.input[self.index..]
    }

    fn is_eof(&self) -> bool {
        self.index >= self.input.len()
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_start(ch)
}

#[cfg(test)]
mod spec {
    use super::*;

    #[derive(Debug)]
    struct ValidCase {
        input: &'static str,
        expected: ParsedFormatSpec,
    }

    fn valid_cases() -> Vec<ValidCase> {
        vec![
            ValidCase {
                input: "",
                expected: ParsedFormatSpec {
                    sign: None,
                    width: None,
                    precision: None,
                },
            },
            ValidCase {
                input: ">8.*",
                expected: ParsedFormatSpec {
                    sign: None,
                    width: Some(Count::Integer(8)),
                    precision: Some(Precision::Asterisk),
                },
            },
            ValidCase {
                input: ">+8.*",
                expected: ParsedFormatSpec {
                    sign: Some(Sign::Plus),
                    width: Some(Count::Integer(8)),
                    precision: Some(Precision::Asterisk),
                },
            },
            ValidCase {
                input: "-.1$x",
                expected: ParsedFormatSpec {
                    sign: Some(Sign::Minus),
                    width: None,
                    precision: Some(Precision::Count(Count::Parameter(Argument::Position(1)))),
                },
            },
            ValidCase {
                input: "a^#043.8?",
                expected: ParsedFormatSpec {
                    sign: None,
                    width: Some(Count::Integer(43)),
                    precision: Some(Precision::Count(Count::Integer(8))),
                },
            },
            ValidCase {
                input: "λ<+12.user$",
                expected: ParsedFormatSpec {
                    sign: Some(Sign::Plus),
                    width: Some(Count::Integer(12)),
                    precision: Some(Precision::Count(Count::Parameter(Argument::Identifier(
                        "user".to_owned(),
                    )))),
                },
            },
            ValidCase {
                input: "name$",
                expected: ParsedFormatSpec {
                    sign: None,
                    width: Some(Count::Parameter(Argument::Identifier("name".to_owned()))),
                    precision: None,
                },
            },
            ValidCase {
                input: "8name",
                expected: ParsedFormatSpec {
                    sign: None,
                    width: Some(Count::Integer(8)),
                    precision: None,
                },
            },
            ValidCase {
                input: "_tmp$.*",
                expected: ParsedFormatSpec {
                    sign: None,
                    width: Some(Count::Parameter(Argument::Identifier("_tmp".to_owned()))),
                    precision: Some(Precision::Asterisk),
                },
            },
            ValidCase {
                input: "5.01$X?",
                expected: ParsedFormatSpec {
                    sign: None,
                    width: Some(Count::Integer(5)),
                    precision: Some(Precision::Count(Count::Parameter(Argument::Position(1)))),
                },
            },
        ]
    }

    #[test]
    fn parses_valid_format_specs_with_regex() {
        for case in valid_cases() {
            let parsed = parse_regex(case.input).unwrap();
            assert_eq!(parsed, case.expected, "input: {}", case.input);
        }
    }

    #[test]
    fn parses_valid_format_specs_with_custom_parser() {
        for case in valid_cases() {
            let parsed = parse_custom(case.input).unwrap();
            assert_eq!(parsed, case.expected, "input: {}", case.input);
        }
    }

    #[test]
    fn both_implementations_match_on_valid_inputs() {
        for case in valid_cases() {
            let regex = parse_regex(case.input).unwrap();
            let custom = parse_custom(case.input).unwrap();
            assert_eq!(regex, custom, "input: {}", case.input);
        }
    }

    #[test]
    fn rejects_invalid_specs_in_both_implementations() {
        let too_large = format!("{}", "9".repeat(100));
        let too_large_precision = format!("1.{}", "9".repeat(100));

        let invalid_inputs = vec![
            ".",
            "++1",
            "-.",
            "_",
            "name$.",
            "name$._",
            "1$$",
            "1.$x",
            "a^#043.",
            &too_large,
            &too_large_precision,
        ];

        for input in invalid_inputs {
            assert!(parse_regex(input).is_err(), "regex should reject {input}");
            assert!(parse_custom(input).is_err(), "custom should reject {input}");
        }
    }

    #[test]
    fn identifies_integer_overflow() {
        let huge = format!("{}", "9".repeat(100));

        let regex_error = parse_regex(&huge).unwrap_err();
        let custom_error = parse_custom(&huge).unwrap_err();

        assert!(matches!(
            regex_error,
            ParseError::InvalidInteger { field: "width", .. }
        ));
        assert!(matches!(
            custom_error,
            ParseError::InvalidInteger { field: "width", .. }
        ));
    }
}
