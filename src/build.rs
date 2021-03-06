use clap::ArgMatches;
use ini::Ini;
use glob::glob;
use tempdir::TempDir;
use std::fs::*;
use std::str;
use std::path::*;
use std::process;
use std::os::unix::fs;
use std::process::Command;
use tar::Archive;
use std::env;
use std::time::Instant;
use console::{style, Emoji};

use indicatif::{HumanDuration, ProgressBar, ProgressStyle};

use crate::config;

static CHECKMARK: Emoji = Emoji("✔️", "✓ ");

pub const BINS: [&'static str; 21] = ["bin/ct_run",
                                      "bin/dialyzer",
                                      "bin/epmd",
                                      "bin/erl",
                                      "bin/erlc",
                                      "bin/escript",
                                      "bin/run_erl",
                                      "bin/run_test",
                                      "bin/to_erl",
                                      "bin/typer",
                                      "lib/erlang/lib/diameter-*/bin/diameterc",
                                      "lib/erlang/lib/edoc-*/priv/edoc_generate",
                                      "lib/erlang/lib/erl_interface-*/bin/erl_call",
                                      "lib/erlang/lib/inets-*/priv/bin/runcgi.sh",
                                      "lib/erlang/lib/observer-*/priv/bin/cdv",
                                      "lib/erlang/lib/observer-*/priv/bin/etop",
                                      "lib/erlang/lib/odbc-*/priv/bin/odbcserver",
                                      "lib/erlang/lib/os_mon-*/priv/bin/memsup",
                                      "lib/erlang/lib/snmp-*/bin/snmpc",
                                      "lib/erlang/lib/tools-*/bin/emem",
                                      "lib/erlang/lib/webtool-*/priv/bin/start_webtool"];

fn latest_tag(repo_dir: &str) -> String {
    let output = Command::new("git")
        .args(&["rev-list", "--tags", "--max-count=1"])
        .current_dir(repo_dir)
        .output()
        .unwrap_or_else(|e| { error!("git rev-list failed: {}", e); process::exit(1) });

    if !output.status.success() {
        error!("finding latest tag of {} failed: {}", repo_dir, String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }

    let rev = str::from_utf8(&output.stdout).unwrap();
    let output = Command::new("git")
        .args(&["describe", "--tags", &rev.trim()])
        .current_dir(repo_dir)
        .output()
        .unwrap_or_else(|e| { error!("git describe failed: {}", e); process::exit(1) });

    if !output.status.success() {
        error!("describing latest tag of {} failed: {}", repo_dir, String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }

    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

pub fn update_bins(bin_path: &Path, links_dir: &Path) {
    let _ = create_dir_all(links_dir);
    for &b in BINS.iter() {
        let f = Path::new(b).file_name().unwrap();
        let link = links_dir.join(f);
        debug!("linking {} to {}", link.display(), bin_path.display());
        let _ = remove_file(&link);
        let _ = fs::symlink(bin_path, link);
    }
}

pub fn tags(sub_m: &ArgMatches, config: Ini) {
    let repo = sub_m.value_of("repo").unwrap_or("default");
    let git_repo = &config::lookup("repos", repo, &config).unwrap();
    let dir = &config::lookup_cache_dir(&config);
    let repo_dir = Path::new(dir).join("repos").join(repo);

    if !repo_dir.exists() {
        info!("Cloning repo {} to {}", git_repo, repo_dir.to_str().unwrap());
        clone_repo(git_repo, repo_dir.to_owned());
    }

    let output = Command::new("git")
        .args(&["tag"])
        .current_dir(repo_dir)
        .output()
        .unwrap_or_else(|e| { error!("git command failed: {}", e); process::exit(1) });

    if !output.status.success() {
        error!("tag failed: {}", String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }

    println!("{}", String::from_utf8_lossy(&output.stdout).trim().to_string());
}


pub fn fetch(sub_m: &ArgMatches, config: Ini) {
    let repo = sub_m.value_of("repo").unwrap_or("default");
    let git_repo = &config::lookup("repos", repo, &config).unwrap();
    let dir = &config::lookup_cache_dir(&config);
    let repo_dir = Path::new(dir).join("repos").join(repo);

    let started = Instant::now();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style.clone());
    pb.enable_steady_tick(10);

    if !repo_dir.exists() {
        pb.set_message(&format!("Cloning repo {} to {}", git_repo, repo_dir.to_str().unwrap()));
        clone_repo(git_repo, repo_dir.to_owned());
        pb.println(format!(" {} Cloning repo {} to {:?}", CHECKMARK, git_repo, repo_dir));
    }

    pb.set_message(&format!("Fetching tags from {}", git_repo));
    let output = Command::new("git")
        .args(&["fetch"])
        .current_dir(repo_dir)
        .output()
        .unwrap_or_else(|e| { error!("git fetch failed: {} {}", dir, e); process::exit(1) });

    if !output.status.success() {
        error!("fetch failed: {}", String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }

    pb.println(format!(" {} Fetching tags from {}", CHECKMARK, git_repo));
    pb.finish_and_clear();
    println!("{} fetch in {}", style("Finished").green().bold(), HumanDuration(started.elapsed()));
}

fn clone_repo(git_repo: &str, repo_dir: std::path::PathBuf) {
    let _ = create_dir_all(&repo_dir);
    let output = Command::new("git")
        .args(&["clone", git_repo, "."])
        .current_dir(&repo_dir)
        .output()
        .unwrap_or_else(|e| { error!("git clone failed: {:?} {}", repo_dir, e); process::exit(1) });

    if !output.status.success() {
        error!("clone failed: {}", String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }
}

pub fn run(bin_path: PathBuf, sub_m: &ArgMatches, config_file: &str, config: Ini) {
    let repo = sub_m.value_of("repo").unwrap_or("default");

    let repo_url = &config::lookup("repos", repo, &config).unwrap();
    let dir = &config::lookup_cache_dir(&config);

    let key = "ERLS_CONFIGURE_OPTIONS";
    let empty_string = &"".to_string();
    let configure_options = match env::var(key) {
        Ok(options) => options.to_owned(),
        _ => config::lookup_with_default("erls", "default_configure_options",
                                         empty_string, &config).to_owned(),
    };
    let links_dir = Path::new(dir).join("bin");
    let repo_dir = Path::new(dir).join("repos").join(repo);
    let repo_dir_str = repo_dir.to_str().unwrap();

    let vsn = match sub_m.value_of("VSN").unwrap() {
        "latest" => latest_tag(repo_dir_str),
        vsn => vsn.to_string()
    };

    let id = sub_m.value_of("id").unwrap_or(&vsn);

    let install_dir = Path::new(dir).join("otps").join(id);
    let install_dir_str = install_dir.to_str().unwrap();

    if !install_dir.exists() {
        build(repo_url, repo_dir_str, install_dir_str, &vsn, &configure_options);
        update_bins(bin_path.as_path(), links_dir.as_path());

        // update config file with new built otp entry
        let dist = install_dir.join("dist");
        config::update(id, dist.to_str().unwrap(), config_file);
    } else {
        error!("Directory for {} already exists: {}", id, install_dir_str);
        error!("If this is incorrect remove that directory.");
        error!("Or provide a different id with -i <id>.");
        process::exit(1);
    }
}

fn run_git(args: Vec<&str>) {
    let output = Command::new("git")
        .args(&args)
        .output()
        .unwrap_or_else(|e| { error!("git command failed: {}", e); process::exit(1) });

    if !output.status.success() {
        error!("clone failed: {}", String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }
}

fn clone(repo: &str, dest: &str) {
    run_git(vec!["clone", repo, dest]);
}

fn checkout(dir: &Path, repo_dir: &str, vsn: &str) {
    let otp_tar = dir.join("otp.tar");
    debug!("otp_tar={}", otp_tar.to_str().unwrap());
    let output = Command::new("git")
        .args(&["archive", "-o", otp_tar.to_str().unwrap(), vsn])
        .current_dir(repo_dir)
        .output()
        .unwrap_or_else(|e| { error!("git archive failed: {}", e); process::exit(1) });

    if !output.status.success() {
        error!("checkout of {} failed: {}", vsn, String::from_utf8_lossy(&output.stderr));
        process::exit(1);
    }

    let mut ar = Archive::new(File::open(otp_tar).unwrap());
    ar.unpack(dir).unwrap();
}

fn setup_links(install_dir: &str) {
    for &b in BINS.iter() {
        let f = Path::new(b).file_name().unwrap();
        let bin = Path::new(install_dir).join("dist").join(b);
        let paths = glob(bin.to_str().unwrap()).unwrap();

        match paths.last() {
            Some(x) => {
                let link = Path::new(install_dir).join(f);
                let _ = fs::symlink(x.unwrap().to_str().unwrap(), link);
            },
            None => debug!("file to link not found: {}", f.to_str().unwrap()),
        }
    }
}

pub fn build(repo_url: &str, repo_dir: &str, install_dir: &str, vsn: &str, configure_options: &str) {
    if !Path::new(repo_dir).is_dir() {
        clone(repo_url, repo_dir);
    }

    let started = Instant::now();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
        .template("{prefix:.bold.dim} {spinner} {wide_msg}");

    let pb = ProgressBar::new_spinner();
    pb.set_style(spinner_style.clone());
    pb.enable_steady_tick(10);

    match TempDir::new("erls") {
        Ok(dir) => {
            pb.set_message(&format!("Checking out {}", vsn));

            checkout(dir.path(), repo_dir, vsn);
            let _ = create_dir_all(repo_dir);
            let _ = create_dir_all(install_dir);

            pb.println(format!(" {} {}", CHECKMARK, format!("Checking out tag {}", vsn)));

            let dist_dir = Path::new(install_dir).join("dist");
            let build_steps: &[(_, &[_])] = &[("./otp_build", &["autoconf"]),
                                              ("./configure", &["--prefix", dist_dir.to_str().unwrap(),
                                                                configure_options]),
                                              ("make", &["-j4"]),
                                              ("make", &["install"])
            ];
            for &(step, args) in build_steps.iter() {
                debug!("Running {} {:?}", step, args);
                pb.set_message(&format!("{} {}", step, args.join(" ")));
                let output = Command::new(step)
                    .args(args)
                    .current_dir(dir.path())
                    .output()
                    .unwrap_or_else(|e| { error!("build failed: {}", e); process::exit(1) });

                debug!("stdout: {}", String::from_utf8_lossy(&output.stdout));
                debug!("stderr: {}", String::from_utf8_lossy(&output.stderr));

                pb.println(format!(" {} {} {}", CHECKMARK, step, args.join(" ")));
            };
        },
        Err(e) => {
            error!("failed creating temp directory for build: {}", e);
        }
    }

    pb.set_message(&format!("{}", "Setting up symlinks"));
    setup_links(install_dir);
    pb.println(format!(" {} {}", CHECKMARK, "Setting up symlinks"));

    pb.finish_and_clear();
    println!("{} build in {}", style("Finished").green().bold(), HumanDuration(started.elapsed()));
}
