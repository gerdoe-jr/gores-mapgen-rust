use ndarray::Array2;
use twmap::AnyTile;

use crate::position::{as_index, Vector2};

#[derive(Clone)]
pub struct Brush {
    texture: Array2<bool>,
    scaled_texture: Option<Array2<bool>>,
}

impl Default for Brush {
    fn default() -> Self {
        Self::new()
    }
}

impl Brush {
    pub fn new() -> Self {
        Self {
            texture: Array2::from_elem((1, 1), true),
            scaled_texture: None,
        }
    }

    pub fn from_texture(texture: Array2<bool>) -> Self {
        Self {
            texture,
            scaled_texture: None,
        }
    }

    pub fn circular(size: usize, circularity: f32) -> Self {
        let circularity = circularity.clamp(0.0, 1.0);
        let center = (size - 1) as f32 / 2.0;

        let min_radius = center; // min radius is from center to nearest border
        let max_radius = f32::sqrt(center * center + center * center); // max radius is from center to corner

        let radius = circularity * min_radius + (1.0 - circularity) * max_radius;

        let mut texture = Array2::from_elem((size, size), false);

        for ((x, y), value) in texture.indexed_iter_mut() {
            let distance = f32::sqrt((x as f32 - center).powi(2) + (y as f32 - center).powi(2));
            *value = distance <= radius;
        }

        Self { texture, scaled_texture: None }
    }

    pub fn apply_scale(&mut self, factor: f32) {
        let (old_width, old_height) = self.texture.dim();
        let width = (old_width as f32 * factor) as usize;
        let height = (old_height as f32 * factor) as usize;

        let mut texture = Array2::from_elem((width, height), false);
        let afactor = 1.0 / factor;

        for ((x, y), elem) in texture.indexed_iter_mut() {
            let old_x = (x as f32 * afactor) as usize;
            let old_y = (y as f32 * afactor) as usize;

            *elem = self.texture[[old_x, old_y]];
        }

        self.scaled_texture = Some(texture);
    }

    pub fn reset_scale(&mut self) {
        self.scaled_texture = None;
    }

    pub fn apply<T: AnyTile>(&self, tiles: &mut Array2<T>, pos: Vector2, tile: T) {
        let used_texture = if let Some(t) = &self.scaled_texture {
            t
        } else {
            &self.texture
        };

        let (width, height) = used_texture.dim();
        let (offx, offy) = (
            (width as f32 / 2.0) as usize,
            (height as f32 / 2.0) as usize,
        );

        let top_left = pos - Vector2::from(vec![offx as f32, offy as f32]);
        for ((x, y), &not_empty) in used_texture.indexed_iter() {
            let real_pos = top_left.clone() + Vector2::from(vec![x as f32, y as f32]);
            if not_empty {
                tiles[as_index(real_pos.view())] = tile;
            }
        }
    }
}
