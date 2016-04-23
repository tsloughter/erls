extern crate ini;

use self::ini::Ini;

pub fn read_config(config_file: &str) -> Ini {
    Ini::load_from_file(config_file).unwrap()
}

pub fn lookup<'a>(section: &str, key: &str, conf: &'a Ini) -> &'a str {
    debug!("reading section {} key {}", section, key);
    let section = conf.section(Some(section)).unwrap();
    section.get(key).unwrap()
}

pub fn update(id: &str, dir: &str, config_file: &str) {
    let mut config = Ini::load_from_file(config_file).unwrap();
    config.with_section(Some("erlangs".to_owned()))
        .set(id, dir);
    config.write_to_file(config_file).unwrap();
}
