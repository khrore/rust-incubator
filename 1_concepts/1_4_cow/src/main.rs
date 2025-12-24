use std::{borrow::Cow, env};

const DEFAULT_PATH: &str = "/etc/app/app.conf";

fn get_config_path() -> Cow<'static, str> {
    // Check for --conf command line argument first (highest priority)
    let args: Vec<String> = env::args().collect();
    if let Some(conf_arg) = args.iter().find(|arg| arg.starts_with("--conf=")) {
        let path = conf_arg.strip_prefix("--conf=").unwrap();
        if path.is_empty() {
            eprintln!("Error: --conf argument cannot be empty");
            std::process::exit(1);
        }
        return Cow::Owned(path.to_string());
    }

    // Check for APP_CONF environment variable
    if let Ok(env_path) = env::var("APP_CONF") {
        if !env_path.is_empty() {
            return Cow::Owned(env_path);
        }
    }

    // Return default path without allocation
    Cow::Borrowed(DEFAULT_PATH)
}

fn main() {
    let config_path = get_config_path();
    println!("{}", config_path);
}
