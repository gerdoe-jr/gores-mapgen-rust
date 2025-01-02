use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
};

use egui_snarl::{InPinId, NodeId, Snarl};
use mapgen_core::{
    brush::Brush,
    generator::Generator,
    map::Map,
    mutations::{walker::straight::StraightWalkerMutation, MutationState, Mutator},
    walker::Walker,
};
use twmap::{GameLayer, Group, Image, Tile, TileFlags, TilesLayer, TwMap};

use crate::components::{
    map::load_image,
    ui::bottom_panel::{ExtractMutation, Titled, UiMutation, UiNode},
};

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum DesignLayer {
    Unhookable,
    Hookable,
    Freeze,
}

impl DesignLayer {
    pub fn is_same(&self, id: u8) -> bool {
        match *self {
            DesignLayer::Unhookable => id == 3,
            DesignLayer::Hookable => id == 1,
            DesignLayer::Freeze => id == 0,
        }
    }
}

pub struct DesignImageInfo {
    path: PathBuf,
    automapper_rule: usize,
}

impl DesignImageInfo {
    pub fn new<P: AsRef<Path>>(path: P, automapper_rule: usize) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            automapper_rule,
        }
    }
}

pub struct DesignInfo {
    image_infos: HashMap<DesignLayer, DesignImageInfo>,
}

impl DesignInfo {
    pub fn new(image_infos: HashMap<DesignLayer, DesignImageInfo>) -> Self {
        Self { image_infos }
    }
}

struct Loop<T> {
    count: Option<usize>,
    mutations: Vec<T>,
}

pub struct GenerationContext {
    generator: Generator,
    current_map: Option<TwMap>,
}

impl GenerationContext {
    pub fn new() -> Self {
        Self {
            generator: Generator::new(),
            current_map: None,
        }
    }

    fn load_mutations_from_snarl(
        &mut self,
        generator_node: NodeId,
        snarl: &mut Snarl<UiNode>,
    ) -> Option<(
        Vec<Loop<Box<dyn Mutator<Brush>>>>,
        Vec<Loop<Box<dyn Mutator<Map>>>>,
        Vec<Loop<Box<dyn Mutator<Walker>>>>,
    )> {
        match snarl[generator_node] {
            UiNode::GeneratorNode => {
                fn get_mutations<M>(
                    generator_node: NodeId,
                    snarl: &mut Snarl<UiNode>,
                ) -> Vec<Loop<<<UiMutation as ExtractMutation<M>>::ExtractType as ExtractMutation<M>>::ExtractType>>
                where
                    UiMutation: ExtractMutation<M>,
                {
                    let pin_in_brush = snarl.in_pin(InPinId {
                        node: generator_node,
                        input: <UiMutation as ExtractMutation<M>>::INPUT,
                    });

                    let mut loops = Vec::new();

                    let mut end = if let Some(&end) = pin_in_brush.remotes.first() {
                        end
                    } else {
                        return vec![];
                    };

                    let mut cur_loop = None;

                    loop {
                        let next_node = &snarl[end.node];

                        let pin_id = InPinId {
                            node: end.node,
                            input: 0,
                        };
                        let pin = snarl.in_pin(pin_id);

                        let unwrapped = pin.remotes.first();
                        match unwrapped {
                            Some(&out_end) => end = out_end,
                            _ => {}
                        }

                        match next_node {
                            UiNode::LoopStartNode(count) => {
                                let mut lp: Loop<<<UiMutation as ExtractMutation<M>>::ExtractType as ExtractMutation<M>>::ExtractType> = cur_loop.take().unwrap();

                                lp.count = *count;
                                lp.mutations.reverse();

                                loops.push(lp);
                            }
                            UiNode::LoopEndNode => {
                                cur_loop = Some(Loop {
                                    count: None,
                                    mutations: vec![],
                                });
                            }
                            UiNode::MutationNode(mutation) => {
                                let m = mutation.extract().unwrap();
                                println!("{}", m.title());

                                cur_loop
                                    .as_mut()
                                    .unwrap()
                                    .mutations
                                    .push(m.extract().unwrap());
                            }
                            _ => unreachable!(),
                        };

                        if unwrapped.is_none() {
                            break;
                        }
                    }

                    loops
                }

                let brush = get_mutations::<Brush>(generator_node, snarl);
                let map = get_mutations::<Map>(generator_node, snarl);
                let walker = get_mutations::<Walker>(generator_node, snarl);

                Some((brush, map, walker))
            }
            _ => None,
        }
    }

    pub fn set_scale_factor(&mut self, scale_factor: f32) {
        self.generator.set_scale_factor(scale_factor);
    }

    pub fn get_scale_factor(&self) -> f32 {
        self.generator.get_scale_factor()
    }

    pub fn generate(
        &mut self,
        snarl: &mut Snarl<UiNode>,
        generator_node: NodeId,
        design: &DesignInfo,
        waypoints: Vec<(f32, f32)>,
    ) {
        let Some((mut brush_mutations, mut map_mutations, mut walker_mutations)) =
            self.load_mutations_from_snarl(generator_node, snarl)
        else {
            return;
        };
        for lp in brush_mutations.iter_mut() {
            for mutation in lp.mutations.iter_mut() {
                mutation.reset();
            }
        }
        for lp in map_mutations.iter_mut() {
            for mutation in lp.mutations.iter_mut() {
                mutation.reset();
            }
        }
        for lp in walker_mutations.iter_mut() {
            for mutation in lp.mutations.iter_mut() {
                mutation.reset();
            }
        }

        self.generator.on_step(move |walker, map, brush| {
            fn mutate_all<T>(mutant: &mut T, loops: &mut Vec<Loop<Box<dyn Mutator<T>>>>) {
                for lp in loops.iter_mut() {
                    if let Some(count) = &mut lp.count {
                        if *count == 0 {
                            continue;
                        } else {
                            *count -= 1;
                            for mutation in lp.mutations.iter_mut() {
                                let state = mutation.mutate(mutant);

                                println!("[state]\t\t[idk   ]\t{:?}", state);

                                if state == MutationState::Processing {
                                    break;
                                }
                            }
                        }
                    } else {
                        let mut last_finished = false;
                        let mut idx = 0;
                        if lp.mutations.is_empty() {
                            continue;
                        }
                        let last = lp.mutations.len() - 1;
                        for mutation in lp.mutations.iter_mut() {
                            let state = mutation.mutate(mutant);
                            let processed = state == MutationState::Processing;

                            if idx == last {
                                last_finished = !processed;
                            }

                            println!("[state]\t\t[idk   ]\t{:?}", state);

                            if processed {
                                break;
                            }

                            idx += 1;
                        }

                        if last_finished {
                            for mutation in lp.mutations.iter_mut() {
                                    mutation.reset();
                            }
                            lp.mutations.first_mut().unwrap().mutate(mutant);
                        }
                    }
                }
            }

            mutate_all(brush, &mut brush_mutations);
            mutate_all(map, &mut map_mutations);
            mutate_all(walker, &mut walker_mutations);
        });

        let mut map = self.generator.generate(waypoints);

        // design
        // weird way to do it but whatever
        // im done

        let image_ids: HashMap<DesignLayer, u16, std::hash::RandomState> = design
            .image_infos
            .iter()
            .map(|(&layer, info)| {
                let image = load_image(info.path.as_path());

                let pos = map.images.iter().position(|i| image.eq(i));
                if let Some(idx) = pos {
                    (layer, idx as u16)
                } else {
                    let idx = map.images.len();

                    map.images.push(image);

                    (layer, idx as u16)
                }
            })
            .collect();

        let shape = map.physics_group().layers[0].shape().unwrap();

        let mut design_group = Group::default();

        design_group.name = "Design".to_owned();

        for (&design, &id) in image_ids.iter() {
            let mut layer = TilesLayer::new((shape.w, shape.h));

            layer.name = match design {
                DesignLayer::Unhookable => "Unhookable".to_owned(),
                DesignLayer::Hookable => "Hookable".to_owned(),
                DesignLayer::Freeze => "Freeze".to_owned(),
            };

            let tiles = layer.tiles.unwrap_mut();

            *tiles = map
                .find_physics_layer::<GameLayer>()
                .as_ref()
                .unwrap()
                .tiles
                .unwrap_ref()
                .map(|elem| Tile::new(design.is_same(elem.id) as u8, TileFlags::empty()));

            layer.image = Some(id);

            design_group.layers.push(twmap::Layer::Tiles(layer));
        }

        map.groups.push(design_group);

        self.current_map = Some(map);

        println!("generated");
    }

    pub fn take_map(&mut self) -> Option<TwMap> {
        self.current_map.take()
    }
}
