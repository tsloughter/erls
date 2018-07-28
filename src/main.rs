#[macro_use]
extern crate clap;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate ansi_term;

use std::env;
use std::path::*;
use std::process;
use clap::App;
use log::{LogRecord, LogLevel, LogLevelFilter};
use env_logger::LogBuilder;
use ansi_term::Colour::{Red, Green, Blue};

mod config;
mod build;
mod erl;

fn handle_command(bin_path: PathBuf) {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let base_dir = match env::home_dir() {
        Some(home) => home.join(".config"),
        None => { error!("no home directory available"); process::exit(1) },
    };
    let default_config = base_dir.join("erls").join("config");
    let config_file = matches.value_of("config").unwrap_or(default_config.to_str().unwrap());
    let config = config::read_config(&config_file);

    match matches.subcommand() {
        ("fetch", Some(sub_m)) => {
            build::fetch(sub_m, config);
        },
        ("tags", Some(sub_m)) => {
            build::tags(sub_m, config);
        },
        ("build", Some(sub_m)) => {
            build::run(base_dir, bin_path, sub_m, config_file, config);
        },
        ("update_links", _) => {
            let dir = &config::lookup("erls", "dir", &config).unwrap();
            let links_dir = Path::new(dir).join("bin");
            build::update_bins(bin_path.as_path(), links_dir.as_path());
        },
        ("list", _) => {
            config::list();
        },
        ("switch", Some(sub_m)) => {
            let id = sub_m.value_of("ID").unwrap();
            config::switch(id);
        },
        ("default", Some(sub_m)) => {
            let id = sub_m.value_of("ID").unwrap();
            config::set_default(id);
        },

        _ => { let _ = App::from_yaml(yaml).print_help(); },
    }
}

fn setup_logging() {
    let format = |record: &LogRecord| {
        let color = if record.level() == LogLevel::Error { Red }
        else if record.level() == LogLevel::Info { Green }
        else { Blue };
        color.paint(format!("==> {}", record.args())).to_string()
    };
    let mut builder = LogBuilder::new();

    let key = "DEBUG";
    let level = match env::var(key) {
        Ok(_) => LogLevelFilter::Debug,
        _ => LogLevelFilter::Info,
    };

    builder.format(format).filter(None, level);
    builder.init().unwrap();
}

fn main() {
    setup_logging();

    let mut args = env::args();
    let binname = args.nth(0).unwrap();
    let f = Path::new(&binname).file_name().unwrap();

    if f.eq("erls") {
        match env::current_exe() {
            Ok(bin_path) => {
                debug!("current bin path: {}", bin_path.display());
                handle_command(bin_path)
            },
            Err(e) => { println!("failed to get current bin path: {}", e); process::exit(1) },
        }
    } else {
        match build::BINS.iter().find(|&&x| f.eq(Path::new(x).file_name().unwrap())) {
            Some(x) => {
                let bin = Path::new(x).file_name().unwrap();
                erl::run(bin.to_str().unwrap(), args);
            },
            None => { error!("No such command: {}", f.to_str().unwrap()); process::exit(1) },
        }
    }
}
