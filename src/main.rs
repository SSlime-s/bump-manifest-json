mod parser;

use std::path::Path;

use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use once_cell::sync::Lazy;

use crate::parser::{parse_json};

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
        }));
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
            Query::Major => self.major += 1,
            Query::Minor => self.minor += 1,
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

    let manifest_path = Path::new("./manifest.json");
    let manifest_str = std::fs::read_to_string(manifest_path).unwrap();
    let mut parsed_json = parse_json(manifest_str).expect("Failed to Parse Json");

    if !parsed_json.has_version() {
        panic!("No version found in manifest.json");
    }

    parsed_json.get_version_mut().bump(query);
    std::fs::write(manifest_path, parsed_json.emb_string()).unwrap();
}
