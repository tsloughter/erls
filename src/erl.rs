use std::env;
use std::path::*;
use std::process;
use std::env::Args;
use std::process::Command;

use config;

pub fn run(bin: &str, args: Args) {
    // no -c argument available in this case
    let base_dir = match env::home_dir() {
        Some(home) => home.join(".erls"),
        None => { error!("no home directory available"); process::exit(1) },
    };
    let default_config = base_dir.join("config");
    let config_file = default_config.to_str().unwrap();

    let config = config::read_config(&config_file);

    let erl_to_use = config::lookup("erls", "default", &config);
    let erl_dir = config::lookup("erlangs", erl_to_use, &config);
    let cmd = Path::new(erl_dir).join("bin").join(bin);

    debug!("running {}", cmd.to_str().unwrap());
    let args2: Vec<_> = args.map(|arg| {
        arg.clone()
    }).collect();
    let _ = Command::new(cmd.to_str().unwrap()).args(&args2).status().unwrap_or_else(|e| {
        error!("failed to execute process: {}", e); process::exit(1)
    });
}
