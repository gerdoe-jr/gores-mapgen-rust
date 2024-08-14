use std::collections::HashMap;

use crate::{kernel::Kernel, position::Vector2};
use ndarray::{s, Array2};
use twmap::{AnyTile, GameTile, Layer, LayerKind, Tele, TileFlags, TilemapLayer, TwMap, Version};

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

    pub fn clear(&mut self) {
        fn clear_layer(layer: &mut impl TilemapLayer) {
            layer.tiles_mut().unwrap_mut().fill(Default::default())
        }

        self.raw.physics_group_mut().layers.iter_mut().map(|layer| clear_layer(layer));
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn reshape(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;

        fn reshape_layer(width: usize, height: usize, layer: &mut impl TilemapLayer) {
            let loaded = layer.tiles_mut().unwrap_mut();
            *loaded = Array2::from_elem((width, height), Default::default());
        }

        self.raw_layers.values_mut().map(|layer| {
            reshape_layer(width, height, layer);
        });
    }

    pub fn apply_kernel(
        &mut self,
        pos: Vector2,
        kernel: &Kernel,
        kind: LayerKind,
        tile: impl AnyTile,
    ) -> bool {
        let offset: usize = kernel.size / 2; // offset of kernel wrt. position (top/left)
        let extend: usize = kernel.size - offset; // how much kernel extends position (bot/right)

        let exceeds_left_bound = pos.x < offset;
        let exceeds_upper_bound = pos.y < offset;
        let exceeds_right_bound = (pos.x + extend) > self.width();
        let exceeds_lower_bound = (pos.y + extend) > self.height();

        if exceeds_left_bound || exceeds_upper_bound || exceeds_right_bound || exceeds_lower_bound {
            return false;
        }

        let root_pos = Vector2::new(pos.x - offset, pos.y - offset);
        for ((kernel_x, kernel_y), kernel_active) in kernel.vector.indexed_iter() {
            let absolute_pos = Vector2::new(root_pos.x + kernel_x, root_pos.y + kernel_y);
            if *kernel_active {
                let current_type = &self.grid[absolute_pos.as_index()];

                let new_type = match current_type {
                    TileTag::Hookable | TileTag::Freeze => Some(tile.clone()),
                    _ => None,
                };

                if let Some(new_type) = new_type {
                    self.grid[absolute_pos.as_index()] = new_type;
                }
            }
        }

        return true;
    }

    pub fn pos_in_bounds(&self, pos: &Vector2) -> bool {
        // we dont have to check for lower bound, because of usize
        pos.x < self.width() && pos.y < self.height()
    }

    pub fn check_area_exists(
        &self,
        top_left: Vector2,
        bot_right: Vector2,
        kind: LayerKind,
        tile: impl AnyTile,
    ) -> Result<bool, &'static str> {
        if !self.pos_in_bounds(&top_left) || !self.pos_in_bounds(&bot_right) {
            return Err("checking area out of bounds");
        }

        let area = self
            .grid
            .slice(s![top_left.x..=bot_right.x, top_left.y..=bot_right.y]);

        Ok(area.iter().any(|&block| block == value))
    }

    pub fn check_area_all(
        &self,
        top_left: Vector2,
        bot_right: Vector2,
        kind: LayerKind,
        tile: impl AnyTile,
    ) -> Result<bool, &'static str> {
        if !self.pos_in_bounds(&top_left) || !self.pos_in_bounds(&bot_right) {
            return Err("checking area out of bounds");
        }
        let area = self
            .grid
            .slice(s![top_left.x..=bot_right.x, top_left.y..=bot_right.y]);

        Ok(area.iter().all(|&block| block == value))
    }

    pub fn count_occurence_in_area(
        &self,
        top_left: Vector2,
        bot_right: Vector2,
        kind: LayerKind,
        tile: impl AnyTile,
    ) -> Result<usize, &'static str> {
        if !self.pos_in_bounds(&top_left) || !self.pos_in_bounds(&bot_right) {
            return Err("checking area out of bounds");
        }
        let area = self
            .grid
            .slice(s![top_left.x..=bot_right.x, top_left.y..=bot_right.y]);

        Ok(area.iter().filter(|&&block| block == value).count())
    }

    pub fn set_area(
        &mut self,
        top_left: Vector2,
        bot_right: Vector2,
        kind: LayerKind,
        tile: impl AnyTile,
    ) {
        // don't check if in bounds, user should check it theirselves

        let mut view = self
            .grid
            .slice_mut(s![top_left.x..=bot_right.x, top_left.y..=bot_right.y]);

        for current_value in view.iter_mut() {
            *current_value = value;
        }
    }

    /// sets the outline of an area define by two positions
    pub fn set_area_border(
        &mut self,
        top_left: Vector2,
        bot_right: Vector2,
        value: TileTag,
    ) {
        let top_right = Vector2::new(bot_right.x, top_left.y);
        let bot_left = Vector2::new(top_left.x, bot_right.y);

        for x in top_left.x..=bot_right.x {
            self.
        }

        self.set_area(top_left, top_right, value, overwrite);
        self.set_area(top_right, bot_right, value, overwrite);
        self.set_area(top_left, bot_left, value, overwrite);
        self.set_area(bot_left, bot_right, value, overwrite);
    }
}
