use std::path::*;
use std::process;
use std::env::Args;
use std::process::Command;

use config;

pub fn run(bin: &str, args: Args) {
    // no -c argument available in this case
    let erl_dir = config::erl_to_use();
    let cmd = Path::new(&erl_dir).join("bin").join(bin);

    debug!("running {}", cmd.to_str().unwrap());
    let args2: Vec<_> = args.map(|arg| {
        arg.clone()
    }).collect();
    let _ = Command::new(cmd.to_str().unwrap()).args(&args2).status().unwrap_or_else(|e| {
        error!("failed to execute process: {}", e); process::exit(1)
    });
}
