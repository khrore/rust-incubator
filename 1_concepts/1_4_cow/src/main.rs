use std::borrow::Cow;
use std::env::{self, Args, args};
use std::path::{Path, PathBuf};

const ENV: &str = "APP_CONF";
const ARG: &str = "--conf";

fn get_config<'a>(path: &'a Path) -> Cow<'a, Path> {
    let mut detected = false;
    for arg in args().skip(1) {
        if detected {
            return Cow::Owned(PathBuf::from(arg));
        } else if arg == ARG {
            detected = true;
        }
    }
    if let Ok(env) = env::var(ENV) {
        return Cow::Owned(PathBuf::from(env));
    }

    Cow::Borrowed(path)
}

fn main() {
    println!(
        "path: {}",
        get_config(Path::new("foo.txt")).to_string_lossy()
    );
    // let path = Path::new("foo.txt");
    // match path.to_string_lossy() {
    //     Cow::Borrowed(_str_ref) => println!("path was valid UTF-8"),
    //     Cow::Owned(_new_string) => println!("path was not valid UTF-8"),
    // }
}
