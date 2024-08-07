use std::{
    collections::HashMap,
    error::Error,
    fs::{self},
    path::Path,
};

use mapgen_core::{
    generator::{Generator, GeneratorParams},
    kernel::Kernel,
    map::{BlockType, Map},
    random::{random_seed, Random, RandomDist},
    walker::{Walker, WalkerParams, Waypoints},
};
use serde::de::DeserializeOwned;

const SILENT: bool = true;
const LAST_SEED: u64 = 100_000; // u64::MAX

fn main() {
    use std::time::Instant;

    let gen = load_configs_from_dir::<GeneratorParams, _>("../data/configs/generator").unwrap()
        ["insaneV2"];

    let wal = load_configs_from_dir::<WalkerParams, _>("../data/configs/walker").unwrap()
        ["insaneV2"]
        .clone();

    let way = load_configs_from_dir::<Waypoints, _>("../data/configs/waypoints").unwrap()
        ["hor_line"]
        .clone();

    let prng = Random::new(
        random_seed(),
        RandomDist::new(wal.shift_weights.clone()),
        RandomDist::new(wal.outer_margin_probs.clone()),
        RandomDist::new(wal.inner_size_probs.clone()),
        RandomDist::new(wal.circ_probs.clone()),
    );

    let mut walker = Walker::new(
        Kernel::new(5, 0.0),
        Kernel::new(7, 0.0),
        prng,
        wal.clone(),
    );

    walker.set_waypoints(way).set_bounds(300, 150);

    let map = Map::new(300, 150, BlockType::Hookable);
    let mut generator = Generator::new(map, walker, gen);

    let now = Instant::now();

    {
        for seed in 0..LAST_SEED {
            if !SILENT {
                print!("processing {}", seed);
            }

            let result = generator.finalize(200_000);

            if !SILENT {
                match result {
                    Err(error) => println!(": {}", error),
                    _ => println!(": success"),
                }
            }

            generator.map.clear();
        }
    }

    let elapsed = now.elapsed();

    println!("elapsed {:.2?}", elapsed);
}

pub fn load_configs_from_dir<C, P>(path: P) -> Result<HashMap<String, C>, Box<dyn Error>>
where
    C: DeserializeOwned,
    P: AsRef<Path>,
{
    let mut configs = HashMap::new();

    for file_path in fs::read_dir(path)? {
        let file_path = file_path?.path();
        let osstr_file_name = file_path.file_name().unwrap(); // it will never be None since "Returns None if the path terminates in .."
        let file_name = osstr_file_name
            .to_str()
            .unwrap() // believe to user that it will be valid utf8, what an asshole will use utf16 for fucking generator config name?
            .replace(".json", "");

        let data = fs::read_to_string(&file_path).unwrap();

        configs.insert(file_name.to_string(), serde_json::from_str::<C>(&data)?);
    }

    Ok(configs)
}
