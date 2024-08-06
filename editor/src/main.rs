pub mod config;
pub mod editor;
pub mod fps_control;
pub mod gui;
pub mod rendering;

use crate::{editor::*, fps_control::*, rendering::*};
use macroquad::{color::*, miniquad, window::*};
use mapgen_core::map::*;
use miniquad::conf::{Conf, Platform};
use simple_logger::SimpleLogger;

const DISABLE_VSYNC: bool = true;

fn window_conf() -> Conf {
    Conf {
        window_title: "mapgen editor".to_owned(),
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
    SimpleLogger::new().init().unwrap();

    let mut editor = Editor::new();
    let mut fps_ctrl = FPSControl::new().with_max_fps(120);

    loop {
        fps_ctrl.on_frame_start();
        editor.on_frame_start();
        editor.define_egui();

        // optionally, start generating next map right away
        if editor.is_paused() && editor.auto_generate {
            editor.set_playing();
        }
        // perform walker steps
        let steps = match editor.instant {
            true => usize::max_value(),
            false => editor.steps_per_frame,
        };

        if editor.generator.as_ref().is_some() {
            if editor.width != editor.generator.as_ref().unwrap().map.width()
                || editor.height != editor.generator.as_ref().unwrap().map.height()
            {
                editor.generator.as_mut().unwrap().reshape(editor.width, editor.height);
            }
    
            for _ in 0..steps {
                if editor.is_paused() || editor.generator.as_ref().unwrap().walker.finished {
                    break;
                }

                editor
                    .generator
                    .as_mut()
                    .unwrap()
                    .step()
                    .unwrap_or_else(|err| {
                        println!("Walker Step Failed: {:}", err);
                        editor.set_setup();
                    });

                // walker did a step using SingleStep -> now pause
                if editor.is_single_setp() {
                    editor.set_stopped();
                }
            }

            // this is called ONCE after map was generated
            if editor.generator.as_ref().unwrap().walker.finished && !editor.is_setup() {
                // kinda crappy, but ensure that even a panic doesnt crash the program
                editor
                    .generator
                    .as_mut()
                    .unwrap()
                    .post_processing()
                    .unwrap_or_else(|err| {
                        println!("Post Processing Failed: {:}", err);
                    });

                // switch into setup mode for next map
                editor.set_setup();
            }

            editor.set_cam();
            editor.handle_user_inputs();

            clear_background(WHITE);

            draw_chunked_grid(
                &editor.generator.as_ref().unwrap().map.grid,
                &editor.generator.as_ref().unwrap().map.chunks_edited,
                editor.generator.as_ref().unwrap().map.chunk_size,
            );
            draw_walker_kernel(
                &editor.generator.as_ref().unwrap().walker,
                KernelType::Outer,
            );
            draw_walker_kernel(
                &editor.generator.as_ref().unwrap().walker,
                KernelType::Inner,
            );
            draw_walker(&editor.generator.as_ref().unwrap().walker);
            draw_waypoints(
                &editor
                    .config
                    .waypoints
                    .get()
                    .with_map_bounds(editor.width, editor.height),
            );
        }

        egui_macroquad::draw();
        fps_ctrl.wait_for_next_frame().await;
    }
}
