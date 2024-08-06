use std::{fs, path::PathBuf};

use clap::{command, crate_version, Parser};
use exporter::{Exporter, ExporterConfig};
use mapgen_core::{
    generator::{Generator, GeneratorParams},
    kernel::Kernel,
    map::{BlockType, Map},
    random::{Random, RandomDist, Seed},
    walker::{Walker, WalkerParams, Waypoints},
};
use twmap::TwMap;

pub mod exporter;

#[derive(Parser, Debug)]
struct ExporterArgs {
    /// debug to console
    #[arg(short, long, default_value_t = false)]
    verbose: bool,

    /// path to base map
    #[arg(long, default_value = "../data/maps/test.map")]
    base_map: PathBuf,

    /// path to generator config
    #[arg(long, default_value = "../data/configs/generator/easy.json")]
    gen_config: PathBuf,

    /// path to walker config
    #[arg(long, default_value = "../data/configs/walker/easy.json")]
    wal_config: PathBuf,

    /// path to waypoints config
    #[arg(long, default_value = "../data/configs/waypoints/hor_line.json")]
    way_config: PathBuf,

    /// path to exporter config
    #[arg(long, default_value = "../data/configs/exproter/default.json")]
    exp_config: PathBuf,

    /// path to exporter config
    #[arg(long, default_value = "./out.map")]
    out: PathBuf,

    /// seed for generation
    #[arg(long, default_value_t = 0xdeadbeef)]
    seed: u64,

    /// max steps of generation
    #[arg(long, default_value_t = 0xb00b)]
    max_steps: usize,
}

#[derive(Parser, Debug)]
#[command(name = "mapgen-exporter")]
#[command(version = crate_version!())]
#[command(about = "Generate and export gores maps", long_about = None)]
enum Command {
    #[clap(
        name = "genex",
        about = "Generate and export gores map with provided configurations"
    )]
    Genex(ExporterArgs),
}

fn main() {
    match Command::parse() {
        Command::Genex(args) => {
            let gen_config_data = fs::read_to_string(args.gen_config)
                .expect("failed to load generator configuration");
            let wal_config_data =
                fs::read_to_string(args.wal_config).expect("failed to load walker configuration");
            let way_config_data = fs::read_to_string(args.way_config)
                .expect("failed to load waypoints configuration");
            let exp_config_data =
                fs::read_to_string(args.exp_config).expect("failed to load exporter configuration");

            let gen: GeneratorParams = serde_json::from_str(&gen_config_data).unwrap();
            let wal: WalkerParams = serde_json::from_str(&wal_config_data).unwrap();
            let way: Waypoints = serde_json::from_str(&way_config_data).unwrap();
            let exp: ExporterConfig = serde_json::from_str(&exp_config_data).unwrap();

            let mut tw_map = TwMap::parse_file(args.base_map).expect("failed to parse base map");
            tw_map.load().expect("failed to load base map");

            let prng = Random::new(
                Seed::from_u64(args.seed),
                RandomDist::new(wal.shift_weights.clone()),
                RandomDist::new(wal.outer_margin_probs.clone()),
                RandomDist::new(wal.inner_size_probs.clone()),
                RandomDist::new(wal.circ_probs.clone()),
            );

            let walker = Walker::new(
                Kernel::new(5, 0.0),
                Kernel::new(7, 0.0),
                way.waypoints,
                prng,
                wal,
            );
            let map = Map::new(500, 500, BlockType::Hookable);

            let mut generator = Generator::new(map, walker, gen);

            generator.finalize(args.max_steps).unwrap();

            let mut exporter = Exporter::new(&mut tw_map, &generator.map, exp);

            exporter.finalize().save_map(&args.out);
        }
    }
}
