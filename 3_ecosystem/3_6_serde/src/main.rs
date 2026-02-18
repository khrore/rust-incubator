use thiserror::Error;

use step_3_6::{
    RequestFormatError, RequestParseError, parse_request_json, request_to_toml, request_to_yaml,
};

const REQUEST_JSON: &str = include_str!("../request.json");

#[derive(Debug, Error)]
enum AppError {
    #[error("{0}")]
    Parse(#[from] RequestParseError),
    #[error("{0}")]
    Format(#[from] RequestFormatError),
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), AppError> {
    let request = parse_request_json(REQUEST_JSON)?;

    let yaml = request_to_yaml(&request)?;
    println!("YAML:");
    println!("{yaml}");

    let toml = request_to_toml(&request)?;
    println!("TOML:");
    println!("{toml}");

    Ok(())
}
