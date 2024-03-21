use gores_mapgen_rust::{editor::*, fps_control::*, grid_render::*, map::*};

use macroquad::{color::*, miniquad, window::*};
use miniquad::conf::{Conf, Platform};

const DISABLE_VSYNC: bool = true;
const STEPS_PER_FRAME: usize = 50;

fn window_conf() -> Conf {
    Conf {
        window_title: "egui with macroquad".to_owned(),
        platform: Platform {
            swap_interval: match DISABLE_VSYNC {
                true => Some(0), // set swap_interval to 0 to disable vsync
                false => None,
            },
            ..Default::default()
        },
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut editor = Editor::new(EditorPlayback::Paused, GenerationConfig::default());
    let mut fps_ctrl = FPSControl::new().with_max_fps(60);
    let mut gen = Generator::new(&editor.config);

    loop {
        fps_ctrl.on_frame_start();
        editor.on_frame_start();

        // walker logic TODO: move this into Generator struct
        if editor.playback.is_not_paused() {
            for _ in 0..STEPS_PER_FRAME {
                // check if walker has reached goal position
                if gen.walker.is_goal_reached() == Some(true) {
                    gen.walker
                        .next_waypoint(&editor.config)
                        .unwrap_or_else(|_| {
                            println!("pause due to error fetching next checkpoint");
                            editor.playback.pause();
                        });
                }

                // randomly mutate kernel
                gen.walker.mutate_kernel(&editor.config, &mut gen.rnd);

                // perform one greedy step
                if let Err(err) = gen.walker.probabilistic_step(&mut gen.map, &mut gen.rnd) {
                    println!("walker step failed: '{:}' - pausing...", err);
                    editor.playback.pause();
                }

                // walker did a step using SingleStep -> now pause
                if editor.playback == EditorPlayback::SingleStep {
                    editor.playback.pause();
                    break; // skip following steps for this frame!
                }
            }
        }

        editor.define_egui(&mut gen);
        editor.set_cam(&gen.map);
        editor.handle_user_inputs(&gen.map);

        clear_background(WHITE);
        draw_grid_blocks(&gen.map.grid);
        draw_waypoints(&editor.config.waypoints);
        draw_walker(&gen.walker);
        draw_walker_kernel(&gen.walker, KernelType::Outer);
        draw_walker_kernel(&gen.walker, KernelType::Inner);

        egui_macroquad::draw();

        fps_ctrl.wait_for_next_frame().await;
    }
}