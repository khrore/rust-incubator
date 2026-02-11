use std::{
    cmp::Ordering,
    env, fmt,
    io::{self, BufRead, Write},
};

fn main() {
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    if let Err(err) = run_app(env::args(), &mut input, &mut output) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

fn run_app<I, R, W>(args: I, input: &mut R, output: &mut W) -> Result<(), AppError>
where
    I: IntoIterator<Item = String>,
    R: BufRead,
    W: Write,
{
    let mut args = args.into_iter();
    args.next();
    let secret_arg = args.next().ok_or(AppError::MissingSecretNumber)?;
    let secret_number =
        parse_secret_number(&secret_arg).map_err(|_| AppError::InvalidSecretNumber)?;
    run_game(secret_number, input, output)
}

fn run_game<R, W>(secret_number: u32, input: &mut R, output: &mut W) -> Result<(), AppError>
where
    R: BufRead,
    W: Write,
{
    writeln!(output, "Guess the number!")?;

    loop {
        writeln!(output, "Please input your guess.")?;

        let guess = match read_guess(input)? {
            Some(n) => n,
            None => continue,
        };

        writeln!(output, "You guessed: {}", guess)?;

        match guess.cmp(&secret_number) {
            Ordering::Less => writeln!(output, "Too small!")?,
            Ordering::Greater => writeln!(output, "Too big!")?,
            Ordering::Equal => {
                writeln!(output, "You win!")?;
                break;
            }
        }
    }

    Ok(())
}

fn read_guess<R: BufRead>(input: &mut R) -> Result<Option<u32>, AppError> {
    let mut guess = String::new();
    let bytes = input.read_line(&mut guess)?;
    if bytes == 0 {
        return Err(AppError::InputEnded);
    }
    Ok(parse_guess(&guess))
}

fn parse_secret_number(input: &str) -> Result<u32, SecretNumberError> {
    input
        .trim()
        .parse()
        .map_err(|_| SecretNumberError::NotANumber)
}

fn parse_guess(input: &str) -> Option<u32> {
    input.trim().parse().ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecretNumberError {
    NotANumber,
}

#[derive(Debug)]
enum AppError {
    MissingSecretNumber,
    InvalidSecretNumber,
    InputEnded,
    Io(io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::MissingSecretNumber => {
                write!(f, "No secret number is specified")
            }
            AppError::InvalidSecretNumber => {
                write!(f, "Secret number is not a number")
            }
            AppError::InputEnded => {
                write!(f, "Input ended before guessing the number")
            }
            AppError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parse_secret_number_trims_and_parses() {
        assert_eq!(parse_secret_number(" 7 "), Ok(7));
    }

    #[test]
    fn parse_secret_number_rejects_invalid() {
        assert_eq!(
            parse_secret_number("nope"),
            Err(SecretNumberError::NotANumber)
        );
    }

    #[test]
    fn parse_secret_number_rejects_overflow() {
        assert_eq!(
            parse_secret_number("4294967296"),
            Err(SecretNumberError::NotANumber)
        );
    }

    #[test]
    fn parse_guess_returns_none_for_invalid() {
        assert_eq!(parse_guess("nope"), None);
    }

    #[test]
    fn run_app_requires_secret_number() {
        let mut input = Cursor::new(b"");
        let mut output = Vec::new();
        let args = vec!["prog".to_string()];
        let result = run_app(args, &mut input, &mut output);
        assert!(matches!(result, Err(AppError::MissingSecretNumber)));
    }

    #[test]
    fn run_app_rejects_non_numeric_secret_number() {
        let mut input = Cursor::new(b"");
        let mut output = Vec::new();
        let args = vec!["prog".to_string(), "abc".to_string()];
        let result = run_app(args, &mut input, &mut output);
        assert!(matches!(result, Err(AppError::InvalidSecretNumber)));
    }

    #[test]
    fn run_game_handles_invalid_guess_then_wins() {
        let mut input = Cursor::new(b"nope\n10\n");
        let mut output = Vec::new();
        run_game(10, &mut input, &mut output).unwrap();
        let stdout = String::from_utf8(output).unwrap();
        assert!(!stdout.contains("You guessed: nope"));
        assert!(stdout.contains("You win!"));
    }

    #[test]
    fn run_game_returns_error_on_eof() {
        let mut input = Cursor::new(b"");
        let mut output = Vec::new();
        let result = run_game(10, &mut input, &mut output);
        assert!(matches!(result, Err(AppError::InputEnded)));
    }
}
