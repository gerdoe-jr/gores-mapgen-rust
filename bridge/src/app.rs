use std::{
    collections::HashMap,
    error::Error,
    fs, panic,
    path::{Path, PathBuf},
};

use mapgen_core::{
    generator::{Generator, GeneratorParams},
    kernel::Kernel,
    map::{BlockType, Map},
    random::{Random, RandomDist, Seed},
    walker::{Walker, WalkerParams, Waypoints},
};
use mapgen_exporter::{Exporter, ExporterConfig};

use clap::{crate_version, Parser};
use itertools::Itertools;
use log::{error, info, warn};
use serde::de::DeserializeOwned;
use twmap::TwMap;

use crate::econ::*;

#[derive(Parser, Debug)]
#[command(name = "DDNet Bridge")]
#[command(version = crate_version!())]
#[command(about = "Detect DDNet-Server votes via econ to trigger map generations", long_about = None)]
enum Command {
    #[clap(name = "start", about = "Start the ddnet bridge")]
    StartBridge(BridgeArgs),

    #[clap(
        name = "list",
        about = "Print a list of available map- & generation configs"
    )]
    ListConfigs(BridgeArgs),
}

#[derive(Parser, Debug)]
struct BridgeArgs {
    /// ec_password
    password: String,

    /// ec_port
    port: u16,

    /// debug to console
    #[arg(short, long, default_value_t = false)]
    debug: bool,

    /// path to maps directory
    maps: PathBuf,

    /// path to base maps directory
    #[arg(default_value = "../data/maps")]
    base_maps: PathBuf,

    /// path to generation configurations directory
    #[arg(default_value = "../data/configs/generator")]
    gen_configs: PathBuf,

    /// path to walker configurations directory
    #[arg(default_value = "../data/configs/walker")]
    wal_configs: PathBuf,

    /// path to walker configurations directory
    #[arg(default_value = "../data/configs/waypoints")]
    way_configs: PathBuf,
}

/// keeps track of the server bridge state
pub struct ServerBridge {
    /// econ connection to game server
    econ: Option<Econ>,

    /// loaded base maps
    base_maps: Vec<TwMap>,

    /// stores all available generation configs
    generator_configs: HashMap<String, GeneratorParams>,

    /// stores all available walker configs
    walker_configs: HashMap<String, WalkerParams>,

    /// stores all available map configs
    waypoints_configs: HashMap<String, Waypoints>,

    /// selected generator config
    current_generator_params: String,

    /// selected walker config
    current_walker_params: String,

    /// selected waypoints
    current_waypoints: String,

    /// stores start arguments
    args: BridgeArgs,

    /// map generator
    generator: Generator,
}

impl ServerBridge {
    fn new(args: BridgeArgs) -> ServerBridge {
        let mut base_maps = Vec::new();

        for path in load_base_maps_paths(args.base_maps.as_path()) {
            let mut tw_map = TwMap::parse_file(path).expect("failed to parse base map");
            tw_map.load().expect("failed to load base map");

            base_maps.push(tw_map);
        }

        let generator_configs =
            load_configs_from_dir::<GeneratorParams, _>(args.gen_configs.as_path()).unwrap();
        let walker_configs =
            load_configs_from_dir::<WalkerParams, _>(args.wal_configs.as_path()).unwrap();
        let waypoints_configs =
            load_configs_from_dir::<Waypoints, _>(args.way_configs.as_path()).unwrap();

        let current_generator_params = generator_configs.iter().last().unwrap().0.clone();
        let current_walker_params = walker_configs.iter().last().unwrap().0.clone();
        let current_waypoints = waypoints_configs.iter().last().unwrap().0.clone();

        let gen = &generator_configs[&current_generator_params];
        let wal = &walker_configs[&current_walker_params];
        let way = &waypoints_configs[&current_waypoints];

        let prng = Random::new(
            Seed::random(),
            RandomDist::new(wal.shift_weights.clone()),
            RandomDist::new(wal.outer_margin_probs.clone()),
            RandomDist::new(wal.inner_size_probs.clone()),
            RandomDist::new(wal.circ_probs.clone()),
        );

        let mut walker = Walker::new(Kernel::new(5, 0.0), Kernel::new(7, 0.0), prng, wal.clone());

        walker.set_waypoints(way.clone()).set_bounds(500, 500);

        let map = Map::new(500, 500, BlockType::Hookable);

        let generator = Generator::new(map, walker, gen.clone());

        ServerBridge {
            econ: None,
            base_maps,
            generator_configs,
            walker_configs,
            waypoints_configs,
            current_generator_params,
            current_walker_params,
            current_waypoints,
            args,
            generator,
        }
    }

    fn start(&mut self) {
        self.econ = Some(
            Econ::connect(&format!("127.0.0.1:{}", self.args.port), 1024).unwrap_or_else(|error| {
                panic!("Failed to establish stream connection: {}", error);
            }),
        );

        info!(auth!("Trying to authenticate..."));

        let password = self.args.password.clone();

        if self.econ_unchecked().auth(&password) {
            info!(auth!("Authentication succeed"));
            self.update_votes();
        } else {
            error!(auth!("Authentication failed, try another password"));
            panic!();
        }

        loop {
            match self.econ_unchecked().read() {
                Ok(()) => {
                    while let Some(line) = &self.econ_unchecked().pop_line() {
                        if line.len() < 22 {
                            warn!(recv!("Incomplete econ line: {}"), line);
                            continue;
                        }

                        info!(recv!("{}"), &line[22..]);

                        self.check_call(line);
                    }
                }
                Err(err) => error!(recv!("{}"), err),
            }
        }
    }

    fn clear_votes(&mut self) {
        self.econ_unchecked().send_rcon_cmd("clear_votes").unwrap();
    }

    fn add_vote(&mut self, desc: &str, command: &str) {
        self.econ_unchecked()
            .send_rcon_cmd(&format!("add_vote \"{}\" \"{}\"", desc, command))
            .unwrap();
    }

    fn update_votes(&mut self) {
        self.clear_votes();

        let mut gap_size = 1;

        let mut gap = || {
            let gap = " ".repeat(gap_size);

            gap_size += 1;

            return gap;
        };

        self.add_vote(
            &format!("Random Map Generator by iMilchshake, v{}", crate_version!()),
            "info",
        );
        self.add_vote(&gap(), "info");

        self.add_vote(
            &format!(
                "Current generator configuration: {}",
                self.current_generator_params
            ),
            "info",
        );
        self.add_vote(
            &format!(
                "Current walker configuration: {}",
                self.current_walker_params
            ),
            "info",
        );
        self.add_vote(
            &format!("Current map layout: {}", self.current_waypoints),
            "info",
        );
        self.add_vote(&gap(), "info");

        self.add_vote("Generate Random Map", "echo call generate");
        self.add_vote(&gap(), "info");

        fn list_available<T>(
            configs: &HashMap<String, T>,
            config_type: &str,
            config_inner_type: &str,
        ) -> Vec<(String, String)> {
            let mut votes = Vec::new();

            for name in configs.keys() {
                votes.push((
                    format!("Set {} configuration: {}", config_type, name),
                    format!("echo call configurate {} {}", config_inner_type, name),
                ));
            }

            return votes;
        }

        for (desc, command) in &list_available(&self.generator_configs, "generator", "generator") {
            self.add_vote(desc, command);
        }

        self.add_vote(&gap(), "info");

        for (desc, command) in &list_available(&self.walker_configs, "walker", "walker") {
            self.add_vote(desc, command);
        }

        self.add_vote(&gap(), "info");

        for (desc, command) in &list_available(&self.waypoints_configs, "layout", "waypoints") {
            self.add_vote(desc, command);
        }
    }

    /// checks whether the econ message regards votes
    fn check_call(&mut self, data: &str) {
        let mut callback_args = Vec::new();

        let mut idx = 0;

        for piece_view in data.split(' ') {
            if idx == 3 {
                // handle only echo
                if piece_view != "console:" {
                    return;
                }
            } else if idx == 4 {
                // handle only "call"
                if piece_view != "call" {
                    return;
                }
            } else if idx > 4 {
                callback_args.push(piece_view);
            }

            idx += 1;
        }

        match callback_args[0] {
            "generate" => {
                let seed = Seed::random();
                let mut map_name = self.generate_map(seed);

                while map_name.is_none() {
                    map_name = self.generate_map(seed);
                }

                self.change_map(&map_name.unwrap());
            }
            "configurate" => {
                if callback_args.len() < 3 {
                    warn!(gen!("Missing arguments on configuration call"));
                    return;
                }

                match callback_args[1] {
                    "generator" => {
                        if !self.generator_configs.contains_key(callback_args[2]) {
                            warn!(
                                gen!("Unknown generator configuration: {}"),
                                callback_args[2]
                            );
                            return;
                        }

                        // TODO: quotation marks?
                        self.current_generator_params = callback_args[2].to_string();

                        self.generator.params =
                            self.generator_configs[&self.current_generator_params].clone();
                    }
                    "walker" => {
                        if !self.walker_configs.contains_key(callback_args[2]) {
                            warn!(gen!("Unknown walker configuration: {}"), callback_args[2]);
                            return;
                        }

                        // TODO: quotation marks?
                        self.current_walker_params = callback_args[2].to_string();

                        self.generator.walker.params =
                            self.walker_configs[&self.current_walker_params].clone();

                        let wal = &self.generator.walker.params;

                        // TODO: move to another config
                        self.generator.walker.prng = Random::new(
                            Seed::random(),
                            RandomDist::new(wal.shift_weights.clone()),
                            RandomDist::new(wal.outer_margin_probs.clone()),
                            RandomDist::new(wal.inner_size_probs.clone()),
                            RandomDist::new(wal.circ_probs.clone()),
                        );
                    }
                    "waypoints" => {
                        if !self.waypoints_configs.contains_key(callback_args[2]) {
                            warn!(
                                gen!("Unknown waypoints configuration: {}"),
                                callback_args[2]
                            );
                            return;
                        }

                        // TODO: quotation marks?
                        self.current_waypoints = callback_args[2].to_string();

                        self.generator
                            .walker
                            .set_waypoints(self.waypoints_configs[&self.current_waypoints].clone())
                            .set_bounds(500, 500);
                    }
                    s => warn!(gen!("Unknown configuration: {}"), s),
                }
            }
            _ => {}
        }

        self.update_votes()
    }

    fn generate_map(&mut self, seed: Seed) -> Option<String> {
        let map_name = format!(
            "{}_{}_{}_{}",
            &self.current_generator_params,
            &self.current_walker_params,
            &self.current_waypoints,
            seed.0
        );

        let map_path = self
            .args
            .maps
            .canonicalize()
            .unwrap()
            .join(map_name.clone() + ".map");

        info!(gen!("Generating {}"), map_name);

        self.generator.map.clear();

        match self.generator.finalize(100_000) {
            Ok(()) => {
                info!(gen!("Finished map generation"));

                let idx = Seed::random().0 as usize % self.base_maps.len();

                let mut exporter = Exporter::new(
                    self.base_maps.get_mut(idx).unwrap(),
                    &self.generator.map,
                    ExporterConfig::default(),
                );

                exporter.finalize().save_map(&map_path);

                info!(gen!("Finished map exporting"));

                return Some(map_name);
            }
            Err(generation_error) => {
                warn!(gen!("Generation Error: {:?}"), generation_error);
            }
        }

        None
    }

    fn change_map(&mut self, map_name: &str) {
        self.econ_unchecked()
            .send_rcon_cmd(&format!("change_map {}", map_name))
            .unwrap();
        self.econ_unchecked().send_rcon_cmd("reload").unwrap();
    }

    pub fn say(&mut self, message: &str) {
        self.econ_unchecked()
            .send_rcon_cmd(&format!("say {message}"))
            .unwrap();
    }

    fn econ_unchecked(&mut self) -> &mut Econ {
        self.econ.as_mut().unwrap()
    }

    pub fn run() {
        match Command::parse() {
            Command::StartBridge(args) => ServerBridge::new(args).start(),
            Command::ListConfigs(args) => print_configs(args),
        }
    }
}

fn print_configs(args: BridgeArgs) {
    println!(
        "GeneratorParams: {}",
        load_configs_from_dir::<GeneratorParams, _>(args.gen_configs.as_path())
            .unwrap()
            .keys()
            .into_iter()
            .join(",")
    );
    println!(
        "WalkerParams: {}",
        load_configs_from_dir::<WalkerParams, _>(args.wal_configs.as_path())
            .unwrap()
            .keys()
            .into_iter()
            .join(",")
    );
    println!(
        "Waypoints: {}",
        load_configs_from_dir::<Waypoints, _>(args.way_configs.as_path())
            .unwrap()
            .keys()
            .into_iter()
            .join(",")
    );
}

fn load_base_maps_paths<P: AsRef<Path>>(path: P) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for file_path in std::fs::read_dir(path).unwrap() {
        let file_path = file_path.unwrap().path();

        paths.push(file_path);
    }

    paths
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
