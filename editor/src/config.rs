use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    path::Path,
};

use mapgen_core::{
    generator::GeneratorParams,
    walker::{WalkerParams, Waypoints},
};
use serde::{de::DeserializeOwned, Serialize};

pub struct Configuration<T: DeserializeOwned> {
    pub current: String,
    pub all: HashMap<String, T>,
}

impl<T: DeserializeOwned> Configuration<T> {
    pub fn new() -> Self {
        Self {
            current: String::new(),
            all: HashMap::new(),
        }
    }

    pub fn get(&self) -> &T {
        self.all.get(&self.current).unwrap()
    }

    pub fn get_mut(&mut self) -> &mut T {
        self.all.get_mut(&self.current).unwrap()
    }
}

pub struct Configurations {
    pub generator: Configuration<GeneratorParams>,
    pub walker: Configuration<WalkerParams>,
    pub waypoints: Configuration<Waypoints>,
}

impl Configurations {
    pub fn new() -> Self {
        Self {
            generator: Configuration::new(),
            walker: Configuration::new(),
            waypoints: Configuration::new(),
        }
    }

    pub fn load_generator<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        self.generator = load_configs_from_dir(path)?;

        Ok(())
    }

    pub fn load_walker<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        self.walker = load_configs_from_dir(path)?;

        Ok(())
    }

    pub fn load_waypoints<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn Error>> {
        self.waypoints = load_configs_from_dir(path)?;

        Ok(())
    }
}

fn load_configs_from_dir<C, P>(path: P) -> Result<Configuration<C>, Box<dyn Error>>
where
    C: DeserializeOwned,
    P: AsRef<Path>,
{
    let mut last = String::new();
    let mut configs = HashMap::new();

    for file_path in fs::read_dir(path)? {
        let file_path = file_path?.path();
        let osstr_file_name = file_path.file_name().unwrap(); // it will never be None since "Returns None if the path terminates in .."
        let file_name = osstr_file_name
            .to_str()
            .unwrap() // believe to user that it will be valid utf8, what an asshole will use utf16 for fucking generator config name?
            .replace(".json", "");

        let data = fs::read_to_string(&file_path).unwrap();

        last = file_name.to_string();

        configs.insert(last.clone(), serde_json::from_str::<C>(&data)?);
    }

    Ok(Configuration {
        current: last,
        all: configs,
    })
}

pub fn save_config<C, P>(config: &C, path: P) -> Result<(), Box<dyn Error>>
where
    C: Serialize,
    P: AsRef<Path>,
{
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, config)?;

    Ok(())
}
