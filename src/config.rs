extern crate ini;

use std::fs::File;
use std::path::*;
use std::process;
use std::env;

use self::ini::Ini;

fn home_config_file() -> String {
    let base_dir = match env::home_dir() {
        Some(home) => home.join(".erls"),
        None => { error!("no home directory available"); process::exit(1) },
    };

    let default_config = base_dir.join("config");

    if !default_config.exists() {
        let mut conf = Ini::new();
        conf.with_section(Some("erls".to_owned()))
            .set("dir", base_dir.to_str().unwrap());
        conf.with_section(Some("repos".to_owned()))
            .set("default", "https://github.com/erlang/otp");
        conf.write_to_file("config").unwrap();
    }

    default_config.to_str().unwrap().to_string()
}

fn home_config() -> Ini {
    let config_file = home_config_file();
    read_config(&config_file)
}

pub fn list() {
    let config = home_config();
    let erls = config.section(Some("erlangs"));
    for s in erls {
        for (k, v) in s {
            println!("{} -> {}", k, v);
        }
    }
}

pub fn erl_to_use() -> String {
    let config = home_config();

    let erl_to_use = match Ini::load_from_file("erls.config") {
        Ok(cwd_config) => {
            debug!("Found ./erls.config");
            match lookup("config", "erlang", &cwd_config) {
                Some(entry) => entry.clone(),
                None => {
                    error!("No Erlang entry found in erls.config");
                    error!("Delete or update the config file");
                    process::exit(1)
                }
            }
        },
        Err(_) => {
            debug!("No ./erls.config found, going to default");
            match lookup("erls", "default", &config) {
                Some(entry) => entry.clone(),
                None => {
                    error!("No default Erlang set. Use `erls default <id>`");
                    process::exit(1)
                }
            }
        }
    };

    debug!("Using Erlang with id {}", erl_to_use);
    match lookup("erlangs", &erl_to_use, &config) {
        Some(erl) => erl.clone(),
        None => {
            error!("No directory found for Erlang with id {} in config", erl_to_use);
            process::exit(1)
        }
    }
}

pub fn read_config(config_file: &str) -> Ini {
    Ini::load_from_file(config_file).unwrap()
}

pub fn lookup<'a>(section: &str, key: &str, conf: &'a Ini) -> Option<&'a String> {
    debug!("reading section {} key {}", section, key);
    let section = conf.section(Some(section)).unwrap();
    section.get(key)
}

pub fn update(id: &str, dir: &str, config_file: &str) {
    let mut config = Ini::load_from_file(config_file).unwrap();
    config.with_section(Some("erlangs".to_owned())).set(id, dir);
    config.write_to_file(config_file).unwrap();
}

pub fn switch(id: &str) {
    let config = home_config();
    match lookup("erlangs", id, &config) {
        Some(_) => {
            let cwd_config = Path::new("erls.config");
            { let _ = File::create(cwd_config); }
            let mut mut_config = Ini::load_from_file("erls.config").unwrap();
            mut_config.with_section(Some("config".to_owned())).set("erlang", id);
            mut_config.write_to_file("erls.config").unwrap();
        }
        None => {
            error!("{} is not a configured Erlang install", id);
            process::exit(1)
        }
    }
}

pub fn set_default(id: &str) {
    let mut config = home_config();
    match lookup("erlangs", &id, &config) {
        Some(_) => {
            info!("Setting {} to default Erlang", id);
            config.with_section(Some("erls".to_owned()))
                .set("default", id);
            let config_file = home_config_file();
            config.write_to_file(&config_file).unwrap();
        },
        None => {
            error!("{} is not a configured Erlang install, can't set it to default", id);
            process::exit(1)
        }
    }
}
