use crate::{cli, error::*};
use failure::ResultExt;
use log::debug;
use serde_derive::{Deserialize, Serialize};
use std::{fs::File, path::Path};

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
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct Profile {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub definitions: Vec<Definition>,
}

// TODO make private and wrap in type that has lazy constructed views
#[derive(Default, Deserialize, Serialize, Clone)]
pub struct Config {
    pub default_profile: Profile,
    pub profiles: Vec<Profile>,
}

impl Config {
    pub fn load<P: AsRef<Path>>(config_file: P) -> Result<Self> {
        serde_json::from_reader(File::open(&config_file)?)
            .with_context(|_| {
                format!(
                    "Failed to parse config file, {}",
                    config_file.as_ref().display()
                )
            })
            .map_err(|error| error.into())
    }

    pub fn save<P: AsRef<Path>>(self, config_file: P) -> Result<()> {
        serde_json::to_writer_pretty(File::create(&config_file)?, &self)
            .with_context(|_| {
                format!(
                    "Failed to stringify config file, {}",
                    config_file.as_ref().display()
                )
            })
            .map_err(|error| error.into())
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
            if cli::prompt(format!("{} is already defined, replace it? ", to_add.name))? {
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

// TODO tests
