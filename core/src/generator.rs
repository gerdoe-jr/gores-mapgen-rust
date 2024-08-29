use twmap::{GameTile, TileFlags, TwMap};

use crate::{
    brush::Brush,
    map::Map,
    position::{from_raw, shift_by_direction},
    walker::Walker,
};

pub struct Generator {
    walker: Walker,
    brush: Brush,
    before_step: Option<Box<dyn FnMut(&mut Walker, &mut Map, &mut Brush)>>,
}

impl Generator {
    pub fn new(scale_factor: f32) -> Self {
        Self {
            walker: Walker::new(scale_factor),
            brush: Brush::new(),
            before_step: None,
        }
    }

    pub fn on_step(&mut self, func: impl FnMut(&mut Walker, &mut Map, &mut Brush) + 'static) {
        self.before_step = Some(Box::new(func));
    }

    pub fn generate(&mut self, waypoints: Vec<(f32, f32)>) -> TwMap {
        // prepare canvas
        let mut map = Map::new();

        let scale_factor = self.walker.get_scale_factor();

        // 1. calculate bounds and enlarge them to let walker freely... walk
        let mut freaky_waypoints = waypoints.clone();

        freaky_waypoints.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

        let normal_width = freaky_waypoints.last().unwrap().0 - freaky_waypoints.first().unwrap().0;

        freaky_waypoints.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let normal_height =
            freaky_waypoints.last().unwrap().1 - freaky_waypoints.first().unwrap().1;

        let approx_width = normal_width * scale_factor;
        let approx_height = normal_height * scale_factor;

        // 2. create map with enlarged bounds
        map.reshape(approx_width as usize + 400, approx_height as usize + 400);
        map.fill_game(GameTile::new(1, TileFlags::empty()));

        // 3. setup initial position
        let mut current_pos = from_raw(waypoints[0], scale_factor);
        current_pos[[0]] += 200.0;
        current_pos[[1]] += 200.0;

        self.walker.set_waypoints(waypoints);

        if let Some(ref mut on_step) = &mut self.before_step {
            on_step(&mut self.walker, &mut map, &mut self.brush);
        }

        // loop thru generation
        while self.walker.step(current_pos.view()) != 0 {
            if let Some(ref mut on_step) = &mut self.before_step {
                on_step(&mut self.walker, &mut map, &mut self.brush);
            }

            shift_by_direction(&mut current_pos, 1.0, self.walker.current_state().direction);

            self.brush.apply(
                map.game_layer().tiles.unwrap_mut(),
                current_pos.clone(),
                GameTile::new(0, TileFlags::empty()),
            );
        }

        // reset our tools
        self.walker.reset();
        self.brush = Brush::new();

        // shrink map
        map.finalize()
    }
}
