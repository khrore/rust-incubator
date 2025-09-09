use regex::Regex;
use std::{convert::From, fmt::Display};

const EMAIL_REGEX_FORMAT: &str = "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$";

#[derive(Debug)]
pub struct InvalidEmailFormat;

impl Display for InvalidEmailFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid email format")
    }
}

#[derive(Debug)]
pub struct EmailString(String);

impl From<EmailString> for String {
    fn from(value: EmailString) -> Self {
        value.0
    }
}

impl TryFrom<String> for EmailString {
    type Error = InvalidEmailFormat;

    fn try_from(s_email: String) -> Result<Self, Self::Error> {
        if Regex::new(EMAIL_REGEX_FORMAT)
            .unwrap()
            .is_match(s_email.as_str())
        {
            Ok(Self(s_email))
        } else {
            Err(InvalidEmailFormat)
        }
    }
}

impl Display for EmailString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
