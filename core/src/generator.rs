use crate::{
    map::{BlockType, Map, Overwrite},
    position::Position,
    post_processing as post,
    walker::Walker,
};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeneratorParams {
    /// (min, max) distance between platforms
    pub platform_distance_bounds: (usize, usize),

    /// maximum distance from empty blocks to nearest non empty block
    pub max_distance: f32,

    /// min distance to next waypoint that is considered reached
    pub waypoint_reached_dist: usize,

    /// (min, max) distance for skips
    pub skip_length_bounds: (usize, usize),

    /// min distance between skips
    pub skip_min_spacing_sqr: usize,

    /// min unconnected freeze obstacle size
    pub min_freeze_size: usize,
}

pub struct Generator {
    pub walker: Walker,
    pub map: Map,
    pub params: GeneratorParams,
}

impl Generator {
    /// derive a initial generator state based on a GenerationConfig
    pub fn new(
        map: Map,
        walker: Walker,
        params: GeneratorParams,
    ) -> Generator {
        Generator {
            walker,
            map,
            params,
        }
    }

    pub fn step(&mut self) -> Result<(), &'static str> {
        // check if walker has reached goal position
        if self
            .walker
            .is_goal_reached(self.params.waypoint_reached_dist)
            == Some(true)
        {
            self.walker.next_waypoint();
        }

        if !self.walker.finished {
            // randomly mutate kernel
            self.walker.mutate_kernel();

            // perform one step
            self.walker
                .probabilistic_step(&mut self.map)?;

            // handle platforms
            self.walker.check_platform(
                &mut self.map,
                self.params.platform_distance_bounds.0,
                self.params.platform_distance_bounds.1,
            )?;
        }

        Ok(())
    }

    pub fn post_processing(&mut self) -> Result<(), &'static str> {
        post::fix_edge_bugs(&mut self.map)?;

        generate_room(&mut self.map, self.walker.initial_pos(), 6, 3, Some(BlockType::Start))?;

        generate_room(
            &mut self.map,
            self.walker.pos,
            4,
            3,
            Some(BlockType::Finish),
        )?;

        if self.params.min_freeze_size > 0 {
            // TODO: Maybe add some alternative function for the case of min_freeze_size=1
            post::remove_freeze_blobs(&mut self.map, self.params.min_freeze_size);
        }

        post::fill_open_areas(&mut self.map, self.params.max_distance);

        post::generate_all_skips(
            &mut self.map,
            self.params.skip_length_bounds,
            self.params.skip_min_spacing_sqr,
        )?;

        Ok(())
    }

    pub fn finalize(&mut self, max_steps: usize) -> Result<(), &'static str> {
        for _ in 0..max_steps {
            if self.walker.finished {
                break;
            }
            self.step()?;
        }

        self.post_processing()?;

        Ok(())
    }
}

pub fn generate_room(
    map: &mut Map,
    pos: Position,
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