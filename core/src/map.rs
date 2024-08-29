use crate::position::{as_index, VectorView2};
use ndarray::Array2;
use twmap::{
    AnyTile, CompressedData, GameLayer, GameTile, Group, Layer, Speedup, Switch, Tele, TileFlags,
    Tune, TwMap, Version,
};

// TileTag::Empty | TileTag::EmptyReserved => 0,
// TileTag::Hookable | TileTag::Platform => 1,
// TileTag::Freeze => 9,
// TileTag::Spawn => 192,
// TileTag::Start => 33,
// TileTag::Finish => 34,

pub struct Map {
    raw: TwMap,
}

impl Map {
    pub fn new() -> Self {
        let mut map = TwMap::empty(Version::DDNet06);
        
        map.info.author = "mapgen".to_string();
        map.info.version = "1.0beta".to_string();
        map.info.license = "CC0".to_string();

        map.groups.push(Group::physics());
        map.groups[0].layers.push(Layer::Game(GameLayer {
            tiles: CompressedData::Loaded(Array2::from_elem(
                (1, 1),
                GameTile::new(0, TileFlags::empty()),
            )),
        }));

        Self { raw: map }
    }

    pub fn width(&self) -> usize {
        let game: &GameLayer = self.raw.find_physics_layer::<GameLayer>().unwrap();

        game.tiles.shape().w
    }

    pub fn height(&self) -> usize {
        let game: &GameLayer = self.raw.find_physics_layer::<GameLayer>().unwrap();

        game.tiles.shape().h
    }

    pub fn game_layer(&mut self) -> &mut GameLayer {
        self.raw.find_physics_layer_mut().unwrap()
    }

    pub fn raw_map_mut(&mut self) -> &mut TwMap {
        &mut self.raw
    }

    pub fn finalize(self) -> TwMap {
        self.raw.lossless_shrink_tiles_layers().unwrap()
    }

    /// clears all the placed tiles
    pub fn reshape(&mut self, width: usize, height: usize) {
        if self.width() == width && self.height() == height {
            return;
        }

        fn reshape_layer<T: AnyTile>(tiles: &mut Array2<T>, width: usize, height: usize) {
            *tiles = Array2::from_elem((width, height), Default::default());
        }

        for layer in self.raw.physics_group_mut().layers.iter_mut() {
            match layer {
                Layer::Game(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Front(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Tele(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Speedup(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Tune(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                _ => {}
            }
        }
    }

    pub fn clear(&mut self) {
        fn clear_layer<T: AnyTile>(tiles: &mut Array2<T>) {
            tiles.fill(Default::default());
        }

        for layer in self.raw.physics_group_mut().layers.iter_mut() {
            match layer {
                Layer::Game(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Front(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Tele(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Speedup(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Tune(l) => clear_layer(l.tiles.unwrap_mut()),
                _ => {}
            }
        }
    }

    pub fn fill_game(&mut self, tile: GameTile) {
        if let Some(layer) = self.raw.find_physics_layer_mut::<GameLayer>() {
            layer.tiles.unwrap_mut().fill(tile);
        }
    }

    pub fn fill_front(&mut self, tile: GameTile) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Front(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_switch(&mut self, tile: Switch) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Switch(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_tele(&mut self, tile: Tele) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tele(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_speedup(&mut self, tile: Speedup) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Speedup(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_tune(&mut self, tile: Tune) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tune(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn set_tile_game(&mut self, pos: VectorView2, tile: GameTile) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Game(layer) = layer {
                layer.tiles.unwrap_mut()[as_index(pos)] = tile;
            }
        });
    }

    pub fn set_tile_front(&mut self, pos: VectorView2, tile: GameTile) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Front(layer) = layer {
                layer.tiles.unwrap_mut()[as_index(pos)] = tile;
            }
        });
    }

    pub fn set_tile_tele(&mut self, pos: VectorView2, tile: Tele) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tele(layer) = layer {
                layer.tiles.unwrap_mut()[as_index(pos)] = tile;
            }
        });
    }

    pub fn set_tile_switch(&mut self, pos: VectorView2, tile: Switch) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Switch(layer) = layer {
                layer.tiles.unwrap_mut()[as_index(pos)] = tile;
            }
        });
    }

    pub fn set_tile_tune(&mut self, pos: VectorView2, tile: Tune) {
        let _ = self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tune(layer) = layer {
                layer.tiles.unwrap_mut()[as_index(pos)] = tile;
            }
        });
    }
}
