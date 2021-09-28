mod git;
mod parser;

use std::path::Path;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use once_cell::sync::Lazy;

use crate::parser::parse_json;

fn create_app<'a, 'b>() -> App<'a, 'b> {
    let app = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("version").index(1).validator(|s: String| {
            if ["minor", "major", "patch"].contains(&s.as_str()) {
                Ok(())
            } else {
                if VERSION_REGEX.is_match(&s) {
                    Ok(())
                } else {
                    Err("Invalid version format".to_string())
                }
            }
        }))
        .arg(
            Arg::with_name("git")
                .short("g")
                .long("git")
                .help("git commit and add tag"),
        )
        .arg(
            Arg::with_name("signature")
                .short("S")
                .help("signature for git commit")
                .requires("git"),
        )
        .arg(
            Arg::with_name("message")
                .short("m")
                .long("message")
                .help("message for git commit")
                .requires("git")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("after-run")
                .short("r")
                .long("run")
                .help("run after version bump (before commit)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("file-path")
                .short("f")
                .long("file")
                .help("file path to version.json")
                .takes_value(true)
                .default_value("manifest.json"),
        );
    app
}

enum Query {
    Version(Version),
    Major,
    Minor,
    Patch,
}

pub struct Version {
    major: u16,
    minor: u16,
    patch: u16,
}
impl Version {
    fn from_str(version: &str) -> Result<Self, String> {
        let v = VERSION_REGEX
            .captures(version)
            .ok_or_else(|| "Invalid version format".to_string())?;
        Ok(Self {
            major: v.get(1).unwrap().as_str().parse().unwrap(),
            minor: v.get(2).unwrap().as_str().parse().unwrap(),
            patch: v.get(3).unwrap().as_str().parse().unwrap(),
        })
    }

    fn bump(&mut self, query: Query) {
        match query {
            Query::Major => {
                self.major += 1;
                self.minor = 0;
                self.patch = 0;
            }
            Query::Minor => {
                self.minor += 1;
                self.patch = 0;
            }
            Query::Patch => self.patch += 1,
            Query::Version(v) => {
                *self = v;
            }
        }
    }
}
impl ToString for Version {
    fn to_string(&self) -> String {
        format!("{}.{}.{}", self.major, self.minor, self.patch)
    }
}

static VERSION_REGEX: Lazy<regex::Regex> =
    Lazy::new(|| regex::Regex::new(r"^(\d+)\.(\d+)\.(\d+)$").unwrap());

fn main() {
    let app = create_app();
    let matches = app.get_matches();

    let arg = matches.value_of("version").unwrap_or("patch");
    let query = match arg {
        "major" => Query::Major,
        "minor" => Query::Minor,
        "patch" => Query::Patch,
        x => Query::Version(Version::from_str(x).unwrap()),
    };

    let file_path = matches.value_of("file-path").unwrap();

    let manifest_path = Path::new(file_path);
    let manifest_str = std::fs::read_to_string(manifest_path).unwrap();
    let mut parsed_json = parse_json(manifest_str).expect("Failed to Parse Json");

    if !parsed_json.has_version() {
        panic!("No version found in manifest.json");
    }

    let before_version = parsed_json.get_version().to_string();
    parsed_json.get_version_mut().bump(query);
    std::fs::write(manifest_path, parsed_json.emb_string()).unwrap();

    if matches.is_present("after-run") {
        let after_run = matches.value_of("after-run").unwrap();
        if cfg!(target_os = "windows") {
            std::process::Command::new("cmd")
                .args(&["/C", after_run])
                .status()
        } else {
            std::process::Command::new("sh")
                .arg("-c")
                .arg(after_run)
                .status()
        }
        .expect("Failed to run after-run");
    }

    if matches.is_present("git") {
        let is_signature = matches.is_present("signature");
        let message = matches.value_of("message").map(|x| x.to_string());
        git::git_commit_and_tag(
            parsed_json.get_version(),
            is_signature,
            message,
            manifest_path.to_str().unwrap(),
        )
        .expect("Failed to commit and tag");
    }

    println!(
        "v{} -> v{}",
        before_version,
        parsed_json.get_version().to_string()
    );
}
