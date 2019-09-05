use crate::error::*;
use failure::ResultExt;
use serde_derive::{Deserialize, Serialize};
use std::{fs::File, path::Path};

#[derive(Deserialize, Serialize)]
pub enum Definition {
    Variable(EnvironmentVariable),
    Group { members: Vec<EnvironmentVariable> },
}

#[derive(Default, Deserialize, Serialize)]
pub struct EnvironmentVariable {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Profile {
    pub name: String,
    pub description: Option<String>,
    pub definitions: Vec<Definition>,
}

// TODO make private and wrap in type that has lazy constructed views
#[derive(Default, Deserialize, Serialize)]
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

    pub fn save<P: AsRef<Path>>(&self, config_file: P) -> Result<()> {
        serde_json::to_writer_pretty(File::create(&config_file)?, self)
            .with_context(|_| {
                format!(
                    "Failed to stringify config file, {}",
                    config_file.as_ref().display()
                )
            })
            .map_err(|error| error.into())
    }

    pub fn profile_mut<'a>(&'a mut self, name: &Option<String>) -> Option<&'a mut Profile> {
        if let Some(name) = name {
            self.profiles
                .iter_mut()
                .find(|profile| profile.name == *name)
        } else {
            Some(&mut self.default_profile)
        }
    }
}

impl Profile {
    pub fn add(&mut self, var: EnvironmentVariable) -> Result<()> {
        // TODO error if var already exists?
        // TODO prompt to replace if already exists?
        self.definitions.push(Definition::Variable(var));
        Ok(())
    }
}

// TODO tests
