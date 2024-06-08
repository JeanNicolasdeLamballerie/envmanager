use std::{collections::HashMap, process::Command, env::var} ;
enum Configurations {
    Transparent,
    Default
}
impl Configurations {
    pub fn new(s :  &str) ->  Self {
    match  s  {
        "t"|"transparent" => Configurations::Transparent,
        _  =>  Configurations::Default,
        }
    }
}
enum Editors {
    VSCode,
    Neovim,
    Neovide
}

fn lunar_envs(cfg : Configurations) -> HashMap<String, String> {
    println!("Adding new  envs...");
    let mut map : HashMap<String, String> = HashMap::new();
    //TODO set a default case instead of unwrapping
    let data = var("APPDATA").unwrap();
    let local_data = var("LOCALAPPDATA").unwrap();
    let temp = var("TEMP").unwrap();
    let xdg_data_home =  option_env!("XDG_DATA_HOME").unwrap_or_else(|| {&data});

    let xdg_config_home =  option_env!("XDG_CONFIG_HOME").unwrap_or_else(||{&local_data});
    let xdg_cache_home =  option_env!("XDG_CACHE_HOME").unwrap_or_else(||{&temp});
    let runtime_from_xdg =  String::from(xdg_data_home)+"\\lunarvim";
   //TODO if (config  to use) => replace with path / else  this
    let config_from_xdg =  String::from(xdg_config_home)+"\\lvim";
    let cache_from_xdg =  String::from(xdg_cache_home)+"\\lvim";
    let base_from_runtime =  String::from(&runtime_from_xdg)+"\\lvim";

 let  dirname = match cfg {
        Configurations::Transparent => "\\vim_configs\\lvim",
        Configurations::Default => "\\vim_configs\\default"
    };

 let lvim_cfg = var("LUNARVIM_CONFIG_DIR").unwrap_or_else(|_| {config_from_xdg + dirname});
    map.insert("XDG_DATA_HOME".to_string(),  xdg_data_home.to_string());
    map.insert("XDG_CONFIG_HOME".to_string(),  xdg_config_home.to_string());
    map.insert("XDG_CACHE_HOME".to_string(),  xdg_cache_home.to_string());
  map.insert("LUNARVIM_RUNTIME_DIR".to_string(),  var("LUNARVIM_RUNTIME_DIR").unwrap_or_else(|_| {runtime_from_xdg}));
  map.insert("LUNARVIM_CONFIG_DIR".to_string(),lvim_cfg);
    map.insert("LUNARVIM_CACHE_DIR".to_string(), var("LUNARVIM_CACHE_DIR").unwrap_or_else(|_| {cache_from_xdg}));
    map.insert("LUNARVIM_BASE_DIR".to_string(), var("LUNARVIM_BASE_DIR").unwrap_or_else(|_| {base_from_runtime}));
    map
}

impl Editors {
    pub fn new(s :  &str) ->  (Self, String) {
    match  s  {
        "nvim" => (Editors::Neovim,  String::from("nvim")),
        "lvim" => (Editors::Neovim,  String::from("lvim")),
        "code"|"vscode" => (Editors::VSCode, String::from("code")),
        _  =>  (Editors::Neovide, String::from("neovide")),
        }
    }
}

/// Where to set the behavior for adding environment variables
fn set_environment(args : crate::Args) -> HashMap<String, String> {
    match Configurations::new(&args.config) {
        //  TODO Change to Configurations::Lunar(Transparent) ->  Give to lunar_envs(Transparent)
        Configurations::Transparent => {
            lunar_envs(Configurations::Transparent)
        },
        Configurations::Default => {
             lunar_envs(Configurations::Default)
        },
    }

}


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
pub fn open(args : crate::Args) {
    let (mut command, is_win) = target_command();
    let command_name = if is_win { "/C" } else { "-c" };
    let (_, cmd) = Editors::new(&args.editor);
    let path = args.path.clone();
     command
        .arg(command_name)
        .envs(&set_environment(args));
     let mut output=  command
        .arg(cmd + " " + &path)
        .spawn()
        .expect("failed to execute process");
output.wait().expect("Error waiting for command ");
}
