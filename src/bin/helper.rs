#[path = "../helper.rs"]
mod helper;

#[path = "../thermal_config.rs"]
mod thermal_config;

fn main() {
    if let Err(error) = helper::run() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}
