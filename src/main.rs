// vim: nomodeline
pub mod database;
pub mod logger;
pub mod models;
pub mod schema;
pub mod ui;
pub mod vim;
// use std::thread::spawn;

use clap::Parser;
use vim::execute_configuration;

/// Simple program to open an editor
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Path to open with the editor
    #[arg()]
    path: Option<String>,
    #[arg(short, long)]
    config: Option<String>,

    #[arg(long, short)]
    id: Option<i32>,

    /// Editor to use
    #[arg(short, long, default_value_t = String::from("neovide"))]
    editor: String,
    /// force clear env.
    #[arg(long, default_value_t = false)]
    clear: bool,
    /// Creation GUI
    #[arg(short, long, default_value_t = false)]
    gui: bool,
}
fn main() {
    let args = Args::parse();
    if args.gui {
        ui::show().unwrap();
        return;
    }
    // let handle = spawn(|| {
    execute_configuration(args);
    // });
    // if handle.join().is_err() {
    // println!("An error occured recovering the thread...");
    // };
}
