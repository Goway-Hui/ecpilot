#[path = "../helper.rs"]
mod helper;

fn main() {
    if let Err(error) = helper::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
