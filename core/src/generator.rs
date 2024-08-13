use crate::{
    map::{BlockType, Map, Overwrite}, position::Vector2, random::{Random, Seed}, walker::Walker
};

pub struct Generator {
    pub walker: Walker,
    pub map: Map,
}

impl Generator {
    pub fn new(seed: Seed, map: Map) -> Generator {
        Generator {
            walker: Walker::new(Random::new(seed)),
            map,
        }
    }

    pub fn reshape(&mut self, width: usize, height: usize) {
        self.map.reshape(width, height);
        self.walker.set_bounds(width, height);
    }

    pub fn step(&mut self) -> bool {
        self.walker.step(&mut self.map)
    }

    pub fn finalize(&mut self) {
        while !self.step() {
            break;
        }

        self.walker.reset();
    }
}

pub fn generate_room(
    map: &mut Map,
    pos: Vector2,
    room_size: i32,
    platform_margin: i32,
    zone_type: Option<BlockType>,
) -> Result<(), &'static str> {
    if !map.pos_in_bounds(&pos.shifted_by(room_size + 2, room_size + 1)?)
        || !map.pos_in_bounds(&pos.shifted_by(room_size + 1, room_size + 1)?)
    {
        return Err("generate room out of bounds");
    }

    // carve room
    map.set_area_border(
        pos.shifted_by(-room_size, -room_size)?,
        pos.shifted_by(room_size, room_size)?,
        BlockType::Empty,
        Overwrite::Force,
    );

    // only reserve - 1 so that when this is used for platforms
    map.set_area(
        pos.shifted_by(-room_size + 1, -room_size + 1)?,
        pos.shifted_by(room_size - 1, room_size - 1)?,
        BlockType::EmptyReserved,
        Overwrite::Force,
    );

    match zone_type {
        Some(zone_type) => {
            // set start/finish line
            map.set_area_border(
                pos.shifted_by(-room_size - 1, -room_size - 1)?,
                pos.shifted_by(room_size + 1, room_size + 1)?,
                zone_type,
                Overwrite::ReplaceNonSolidForce,
            );

            // set spawns
            if zone_type == BlockType::Start {
                map.set_area(
                    pos.shifted_by(-(room_size - platform_margin), room_size - 1)?,
                    pos.shifted_by(room_size - platform_margin, room_size - 1)?,
                    BlockType::Spawn,
                    Overwrite::Force,
                );

                map.set_area(
                    pos.shifted_by(-(room_size - platform_margin), room_size + 1)?,
                    pos.shifted_by(room_size - platform_margin, room_size + 1)?,
                    BlockType::Platform,
                    Overwrite::Force,
                );
            }
        }
        None => {
            // for non start/finish rooms -> place center platform
            map.set_area(
                pos.shifted_by(-(room_size - platform_margin), room_size - 3)?,
                pos.shifted_by(room_size - platform_margin, room_size - 3)?,
                BlockType::Platform,
                Overwrite::Force,
            );
        }
    }

    Ok(())
}
