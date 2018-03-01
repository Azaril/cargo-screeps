use std::{fs, io, path::Path};

use {clap, failure, fern, log, toml};

pub enum CliState {
    Check,
    Build,
    BuildUpload,
}

pub fn setup_cli() -> Result<CliState, failure::Error> {
    let cargo_args = clap::App::new("cargo screeps")
        .version(crate_version!())
        .bin_name("cargo")
        .author("David Ross")
        .subcommand(
            clap::SubCommand::with_name("screeps")
                .arg(
                    clap::Arg::with_name("verbose")
                        .short("v")
                        .long("verbose")
                        .multiple(true),
                )
                .arg(
                    clap::Arg::with_name("build")
                        .short("b")
                        .long("build")
                        .takes_value(false)
                        .help("build files, put in target/ in project root"),
                )
                .arg(
                    clap::Arg::with_name("check")
                        .short("c")
                        .long("check")
                        .takes_value(false)
                        .help("runs 'cargo check' with appropriate target"),
                )
                .arg(
                    clap::Arg::with_name("upload")
                        .short("u")
                        .long("upload")
                        .takes_value(false)
                        .help("upload files to screeps (implies build)"),
                )
                .group(
                    clap::ArgGroup::with_name("command")
                        .args(&["build", "upload", "check"])
                        .multiple(false)
                        .required(true),
                ),
        )
        .get_matches();

    let args = cargo_args.subcommand_matches("screeps").ok_or_else(|| {
        format_err!("expected first subcommand to be 'screeps' (please run as 'cargo screeps')")
    })?;

    let verbosity = match args.occurrences_of("verbose") {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    fern::Dispatch::new()
        .level(verbosity)
        .format(|out, message, record| out.finish(format_args!("{}: {}", record.target(), message)))
        .chain(io::stdout())
        .apply()
        .unwrap();

    assert!(args.is_present("check") || args.is_present("build") || args.is_present("upload"));

    let state = if args.is_present("check") {
        CliState::Check
    } else if args.is_present("upload") {
        CliState::BuildUpload
    } else {
        CliState::Build
    };

    Ok(state)
}

fn default_hostname() -> String {
    "screeps.com".to_owned()
}

fn default_ptr() -> bool {
    false
}
fn default_branch() -> String {
    "default".to_owned()
}

#[derive(Deserialize)]
struct FileConfiguration {
    username: String,
    password: String,
    #[serde(default = "default_branch")]
    branch: String,
    #[serde(default = "default_hostname")]
    hostname: String,
    #[serde(default)]
    ssl: Option<bool>,
    port: Option<i32>,
    #[serde(default = "default_ptr")]
    ptr: bool,
}

// separate structure so we can have defaults based off of other config values

#[derive(Debug, Clone)]
pub struct Configuration {
    pub username: String,
    pub password: String,
    pub branch: String,
    pub hostname: String,
    pub ssl: bool,
    pub port: i32,
    pub ptr: bool,
}

impl Configuration {
    pub fn setup(root: &Path) -> Result<Self, failure::Error> {
        let config_file = root.join("screeps.toml");
        ensure!(
            config_file.exists(),
            "expected screeps.toml to exist in {}",
            root.display()
        );

        let file_config = toml::from_str(&fs::read_string(config_file)?)?;

        let FileConfiguration {
            username,
            password,
            branch,
            hostname,
            ssl,
            port,
            ptr,
        } = file_config;

        let ssl = ssl.unwrap_or_else(|| hostname == "screeps.com");
        let port = port.unwrap_or_else(|| if ssl { 443 } else { 80 });

        Ok(Configuration {
            username,
            password,
            branch,
            hostname,
            ssl,
            port,
            ptr,
        })
    }
}
