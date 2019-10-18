use crate::{cli, error::*};
use failure::ResultExt;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::{fs::File, io::prelude::*, path::Path};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Definition {
    Variable(EnvironmentVariable),
    Group {
        name: String,
        members: Vec<EnvironmentVariable>,
    },
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct EnvironmentVariable {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub definitions: Vec<Definition>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Config {
    pub default_profile: Profile,
    pub profiles: Vec<Profile>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(config_file: P) -> Result<Self> {
        Config::load_from(File::open(&config_file)?)
            .with_context(|_| {
                format!(
                    "Failed to parse config file, {}",
                    config_file.as_ref().display()
                )
            })
            .map_err(|error| error.into())
    }

    fn load_from<R: Read>(reader: R) -> Result<Self> {
        serde_json::from_reader(reader).map_err(|error| error.into())
    }

    pub fn save<P: AsRef<Path>>(self, config_file: P) -> Result<()> {
        self.save_to(File::create(&config_file)?)
            .with_context(|_| {
                format!(
                    "Failed to stringify config file, {}",
                    config_file.as_ref().display()
                )
            })
            .map_err(|error| error.into())
    }

    fn save_to<W: Write>(self, writer: W) -> Result<()> {
        serde_json::to_writer_pretty(writer, &self).map_err(|error| error.into())
    }

    pub fn profile<'a>(&'a self, name: &Option<String>) -> Option<&'a Profile> {
        if let Some(name) = name {
            self.profiles.iter().find(|profile| profile.name == *name)
        } else {
            Some(&self.default_profile)
        }
    }

    pub fn update_default_profile(self, default_profile: Profile) -> Config {
        Config {
            default_profile,
            ..self
        }
    }
    pub fn upsert_profile(self, to_upsert: Profile) -> Config {
        let mut profiles: Vec<Profile> = self
            .profiles
            .iter()
            .filter(|profile| profile.name != to_upsert.name)
            .map(Clone::clone)
            .collect();
        profiles.push(to_upsert);
        Config { profiles, ..self }
    }
}

impl Profile {
    pub fn add(self, to_add: EnvironmentVariable) -> Result<Profile> {
        let already_defined = self.definitions.iter().any(|var| {
            if let Definition::Variable(var) = var {
                var.name == to_add.name
            } else {
                false
            }
        });
        debug!("{} is already defined? {}", to_add.name, already_defined);
        if already_defined {
            if cli::answer_yes_no(format!("{} is already defined, replace it? ", to_add.name))? {
                debug!("Replacing existing {}", to_add.name);
                let mut definitions: Vec<Definition> = self
                    .definitions
                    .iter()
                    .filter(|var| {
                        if let Definition::Variable(var) = var {
                            var.name != to_add.name
                        } else {
                            true
                        }
                    })
                    .map(Clone::clone)
                    .collect();
                definitions.push(Definition::Variable(to_add));
                Ok(Profile {
                    definitions,
                    ..self
                })
            } else {
                debug!("Using existing {}", to_add.name);
                Ok(self)
            }
        } else {
            let mut definitions = self.definitions.clone();
            definitions.push(Definition::Variable(to_add));
            Ok(Profile {
                definitions,
                ..self
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;

    const DEFAULT_JSON: &str = r#"{
  "default_profile": {
    "name": "",
    "definitions": []
  },
  "profiles": []
}"#;

    #[test]
    fn test_invalid_config() {
        let config = Config::load_from(Cursor::new(String::from("{}").as_bytes()));
        assert!(config.is_err(), "Should not have loaded invalid config!");
    }

    #[test]
    fn test_valid_config() {
        let config = Config::load_from(Cursor::new(DEFAULT_JSON.as_bytes()));
        assert!(
            config.is_ok(),
            "Should have loaded invalid config! {:?}",
            config
        );
    }

    #[test]
    fn test_save() {
        let config = Config::default();
        let mut buffer = Vec::new();
        let cursor = Cursor::new(&mut buffer);
        let result = config.save_to(cursor);
        assert!(result.is_ok(), "Could not save. {:?}", result);
        assert_eq!(String::from_utf8_lossy(&buffer), DEFAULT_JSON);
    }
}
