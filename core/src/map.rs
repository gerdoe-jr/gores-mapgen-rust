use std::collections::HashMap;

use crate::{brush::Brush, position::Vector2};
use ndarray::{s, Array2};
use twmap::{edit::EditTile, AnyTile, GameTile, Layer, LayerKind, Speedup, Switch, Tele, TileFlags, TilemapLayer, Tune, TwMap, Version};

// TileTag::Empty | TileTag::EmptyReserved => 0,
// TileTag::Hookable | TileTag::Platform => 1,
// TileTag::Freeze => 9,
// TileTag::Spawn => 192,
// TileTag::Start => 33,
// TileTag::Finish => 34,

pub struct Map {
    width: usize,
    height: usize,
    raw: TwMap
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            raw: TwMap::empty(Version::DDNet06)
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn reshape(&mut self, width: usize, height: usize) {
        if self.width == width && self.height == height {
            return;
        }

        self.width = width;
        self.height = height;

        fn reshape_layer<T: AnyTile>(tiles: &mut Array2<T>, width: usize, height: usize) {
            *tiles = Array2::from_elem((width, height), Default::default());
        }

        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            match layer {
                Layer::Game(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Front(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Tele(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Speedup(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                Layer::Tune(l) => reshape_layer(l.tiles.unwrap_mut(), width, height),
                _ => {}
            }
        });
    }

    pub fn clear(&mut self) {
        fn clear_layer<T: AnyTile>(tiles: &mut Array2<T>) {
            tiles.fill(Default::default());
        }

        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            match layer {
                Layer::Game(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Front(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Tele(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Speedup(l) => clear_layer(l.tiles.unwrap_mut()),
                Layer::Tune(l) => clear_layer(l.tiles.unwrap_mut()),
                _ => {}
            }
        });
    }
    
    pub fn fill_game(&mut self, tile: GameTile) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Game(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_front(&mut self, tile: GameTile) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Front(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }
    

    pub fn fill_switch(&mut self, tile: Switch) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Switch(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_tele(&mut self, tile: Tele) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tele(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_speedup(&mut self, tile: Speedup) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Speedup(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn fill_tune(&mut self, tile: Tune) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tune(layer) = layer {
                layer.tiles.unwrap_mut().fill(tile);
            }
        });
    }

    pub fn __fill_layer(&mut self, kind: LayerKind, tile: impl AnyTile) {
        self.raw.edit_tiles();
    }

    pub fn fill_layer<T: AnyTile, L: TilemapLayer<TileType=T>>(&mut self, layer: &mut L, tile: T) {
        layer.tiles_mut().unwrap_mut().fill(tile)
    }

    pub fn set_tile_game(&mut self, pos: Vector2, tile: GameTile) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Game(layer) = layer {
                layer.tiles.unwrap_mut()[pos.as_index()] = tile;
            }
        });
    }

    pub fn set_tile_front(&mut self, pos: Vector2, tile: GameTile) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Front(layer) = layer {
                layer.tiles.unwrap_mut()[pos.as_index()] = tile;
            }
        });
    }

    pub fn set_tile_tele(&mut self, pos: Vector2, tile: Tele) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tele(layer) = layer {
                layer.tiles.unwrap_mut()[pos.as_index()] = tile;
            }
        });
    }

    pub fn set_tile_switch(&mut self, pos: Vector2, tile: Switch) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Switch(layer) = layer {
                layer.tiles.unwrap_mut()[pos.as_index()] = tile;
            }
        });
    }

    pub fn set_tile_tune(&mut self, pos: Vector2, tile: Tune) {
        self.raw.physics_group_mut().layers.iter_mut().map(|layer| {
            if let Layer::Tune(layer) = layer {
                layer.tiles.unwrap_mut()[pos.as_index()] = tile;
            }
        });
    }
}

struct TwMapEditFillLayer;

impl EditTile for TwMapEditFillLayer {
    fn game_tile(_tile: &mut GameTile) {}

    fn tele(_tele: &mut Tele) {}

    fn speedup(_speedup: &mut Speedup) {}

    fn switch(_switch: &mut Switch) {}

    fn tune(_tune: &mut Tune) {}
}