use std::{collections::HashMap, env::var, process::Command};

use diesel::SqliteConnection;

use crate::{
    database::{establish_connection, get_config, get_single_executable},
    models::{Executable, LinkedConfiguration},
};
//
// enum Configurations {
//     Lunar(LunarConfigurations),
//     Neovim(NeoConfigurations),
// }
//
// impl Configurations {
//     pub fn new(s: &str) -> Self {
//         match s {
//             "t" | "transparent" => Configurations::Lunar(LunarConfigurations::Transparent),
//             "c" | "chill" => Configurations::Lunar(LunarConfigurations::Chill),
//             "n" | "nvim" => Configurations::Neovim(NeoConfigurations::Default),
//             "lazy" | "lazyvim" => Configurations::Neovim(NeoConfigurations::Lazy),
//             _ => Configurations::Lunar(LunarConfigurations::Default),
//         }
//     }
// }
//
// enum NeoConfigurations {
//     Default,
//     Lazy,
// }
// //
// enum LunarConfigurations {
//     Transparent,
//     Chill,
//     Default,
// }
//
// enum Editors {
//     VSCode,
//     Neovim,
//     Neovide,
// }
type Envs = HashMap<String, String>;
fn get_components(conn: &mut SqliteConnection, cfg: &LinkedConfiguration) -> (Envs, Executable) {
    let envs = cfg.get_environments();
    let executable = get_single_executable(conn, cfg.configuration.exec).unwrap();
    (envs, executable)
}

pub fn execute_configuration(args: crate::Args) {
    let (mut command, is_win) = target_command();
    let command_name = if is_win { "/C" } else { "-c" };
    let mut conn = establish_connection();
    let config = get_config(&mut conn, args.id, args.config).unwrap();
    if config.is_empty() {
        println!("No results found...");
        panic!("Ending process... No configuration found.");
    }
    let (envs, exe) = get_components(&mut conn, &config[0]);
    let path = &args.path;
    let start = match path {
        Some(path) => exe.executable + " " + path,
        None => exe.executable,
    };
    drop(conn);
    command.arg(command_name).envs(envs);
    let mut output = command
        .arg(&start)
        .spawn()
        .expect("failed to execute process");
    output.wait().expect("Error waiting for command ");
}

// TODO move to linux/windows build
// fn lunar_envs(cfg: LunarConfigurations) -> HashMap<String, String> {
//     let mut map: HashMap<String, String> = HashMap::new();
//     //TODO set a default case instead of unwrapping
//     let data = var("APPDATA").unwrap();
//     let local_data = var("LOCALAPPDATA").unwrap();
//     let temp = var("TEMP").unwrap();
//     let xdg_data_home = option_env!("XDG_DATA_HOME").unwrap_or_else(|| &data);
//     let xdg_config_home = option_env!("XDG_CONFIG_HOME").unwrap_or_else(|| &local_data);
//     let xdg_cache_home = option_env!("XDG_CACHE_HOME").unwrap_or_else(|| &temp);
//
//     let runtime_from_xdg = String::from(xdg_data_home) + "\\lunarvim";
//     let config_from_xdg = String::from(xdg_config_home); // Configuration files from the
//                                                          // user
//     let cache_from_xdg = String::from(xdg_cache_home) + "\\lvim";
//     let base_from_runtime = String::from(&runtime_from_xdg) + "\\lvim";
//
//     let dirname = match cfg {
//         LunarConfigurations::Transparent => "\\vim_configs\\lvim",
//         LunarConfigurations::Chill => "\\vim_configs\\chill",
//         LunarConfigurations::Default => "\\vim_configs\\default",
//     };
//
//     let lvim_cfg = config_from_xdg + dirname;
//     map.insert("NVIM_APPNAME".to_string(), "lunar".to_string());
//     map.insert("XDG_DATA_HOME".to_string(), xdg_data_home.to_string());
//     map.insert("XDG_CONFIG_HOME".to_string(), xdg_config_home.to_string());
//     map.insert("XDG_CACHE_HOME".to_string(), xdg_cache_home.to_string());
//     map.insert(
//         "LUNARVIM_RUNTIME_DIR".to_string(),
//         var("LUNARVIM_RUNTIME_DIR").unwrap_or(runtime_from_xdg),
//     );
//     map.insert("LUNARVIM_CONFIG_DIR".to_string(), lvim_cfg);
//     map.insert(
//         "LUNARVIM_CACHE_DIR".to_string(),
//         var("LUNARVIM_CACHE_DIR").unwrap_or(cache_from_xdg),
//     );
//     map.insert(
//         "LUNARVIM_BASE_DIR".to_string(),
//         var("LUNARVIM_BASE_DIR").unwrap_or(base_from_runtime),
//     );
//     println!("Using lunarvim configuration : {}", dirname);
//     map
// }
//
// impl Editors {
//     pub fn new(s: &str) -> (Self, String) {
//         match s {
//             "nvim" => (Editors::Neovim, String::from("nvim")),
//             "lvim" => (Editors::Neovim, String::from("lvim")),
//             "code" | "vscode" => (Editors::VSCode, String::from("code")),
//             _ => (Editors::Neovide, String::from("neovide")),
//         }
//     }
// }
//
/// Where to set the behavior for adding environment variables
// fn set_environment(args: crate::Args) -> HashMap<String, String> {
//     match Configurations::new(&args.config) {
//         Configurations::Lunar(cfg) => lunar_envs(cfg),
//         Configurations::Neovim(cfg) => neo_envs(cfg),
//     }
// }
//
fn target_command() -> (Command, bool) {
    let mut is_win = false;
    let output = if cfg!(target_os = "windows") {
        is_win = true;
        Command::new("pwsh")
    } else {
        Command::new("sh")
    };
    (output, is_win)
}
// pub fn open(args: crate::Args) {
//     let (mut command, is_win) = target_command();
//     let command_name = if is_win { "/C" } else { "-c" };
//     let (_, cmd) = Editors::new(&args.editor);
//     let path = args.path.clone();
//     let envs = set_environment(args);
//     let start = match path {
//         Some(path) => cmd + " " + &path,
//         None => cmd,
//     };
//     command.arg(command_name).envs(envs);
//     let mut output = command
//         .arg(&start)
//         .spawn()
//         .expect("failed to execute process");
//     output.wait().expect("Error waiting for command ");
// }
