use ndarray::Array2;
use twmap::LayerKind;

use crate::{map::{Map, MapElement}, position::Vector2};

#[derive(Clone)]
pub struct Brush {
    texture: Array2<bool>,
}

impl Brush {
    pub fn from_texture(texture: Array2<bool>) -> Self {
        Self { texture }
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

        Self { texture }
    }
}

impl MapElement for Brush {
    fn apply(&mut self, pos: Vector2, map: &mut Map, kind: LayerKind) -> bool {
        let (width, height) = self.texture.dim();
        let (offx, offy) = ((width as f32 / 2.0) as usize, (height as f32 / 2.0) as usize);

        let top_left = Vector2::new(pos.x - offx, pos.y - offy);
        for ((x, y), &not_empty) in self.texture.indexed_iter() {
            let real_pos = Vector2::new(top_left.x + x, top_left.y + y);
            if not_empty {
                // set tile, but how?
            }
        }

        return true;
    }
    
    fn apply_destructable(&mut self, pos: Vector2, map: &mut Map, kind: twmap::LayerKind) -> bool {
        todo!()
    }
}