pub mod vim;
use std::thread::spawn;

use clap::Parser;
use vim::open;

/// Simple program to open an editor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to open with the editor
    #[arg()]
    path: Option<String>,
    #[arg(short, long, default_value_t = String::from("default"))]
    config: String,

    /// Editor to use
    #[arg(short, long, default_value_t = String::from("neovide"))]
    editor: String,
}
fn main() {
    let args = Args::parse();
    let handle = spawn(|| {
        open(args);
    });
    if handle.join().is_err() {
        println!("An error occured recovering the thread...");
    };
}
