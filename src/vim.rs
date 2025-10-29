use std::{collections::HashMap, process::Command};

use diesel::SqliteConnection;

use crate::{
    database::{establish_connection, get_config, get_single_executable},
    models::{Executable, LinkedConfiguration},
};
type Envs = HashMap<String, String>;
fn get_components(conn: &mut SqliteConnection, cfg: &LinkedConfiguration) -> (Envs, Executable) {
    let envs = cfg.get_environments();
    let executable = get_single_executable(conn, cfg.configuration.exec).unwrap();
    (envs, executable)
}

pub fn execute_configuration(args: crate::Args) {
    // let (mut command, is_win) = target_command();
    // let command_name = if is_win { "/C" } else { "-c" };
    let mut conn = establish_connection();
    let config = get_config(&mut conn, args.id, args.config).unwrap();
    if config.is_empty() {
        println!("No results found...");
        panic!("Ending process... No configuration found.");
    }
    let (envs, exe) = get_components(&mut conn, &config[0]);
    let mut command = Command::new(exe.executable);
    // let path = &args.path;
    // let start = match path {
    //     Some(path) => exe.executable + " " + path,
    //     None => exe.executable,
    // };
    drop(conn);
    if args.clear {
        command.env_remove("TERM");
        // command.env_clear();
    }
    if let Some(path) = args.path {
        command.arg(&path);
    }
    command.envs(envs);
    let mode: &str = &exe.mode;
    match mode {
        "wait" => {
            let _ = command
                .spawn()
                .expect("failed to execute process")
                .wait()
                .expect("Error waiting for command ");
        }
        "detach" => {
            #[allow(clippy::zombie_processes)] // We are intentionally calling another process then
            // exiting. We just want to make sure the process was started.
            let _ = command.spawn().expect("failed to execute process");
        }
        _ => {
            panic!("Unknown mode of operation (maybe unimplemented ?)");
        }
    }
}
