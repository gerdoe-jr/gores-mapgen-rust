use macroquad::color::colors;
use macroquad::color::Color;
use macroquad::shapes::*;
use mapgen_core::{map::TileTag, map::KernelType, position::Vector2, walker::Walker};
use ndarray::Array2;

fn blocktype_to_color(value: &TileTag) -> Color {
    match value {
        TileTag::Hookable => colors::BROWN,
        TileTag::Freeze => Color::new(0.0, 0.0, 0.0, 0.8),
        TileTag::Empty => Color::new(0.0, 0.0, 0.0, 0.0),
        TileTag::EmptyReserved => Color::new(0.3, 0.0, 0.0, 0.1),
        TileTag::Finish => Color::new(1.0, 0.1, 0.1, 0.8),
        TileTag::Start => Color::new(0.1, 1.0, 0.1, 0.8),
        TileTag::Platform => Color::new(0.5, 0.5, 0.0, 0.8),
        TileTag::Spawn => Color::new(0.2, 0.2, 0.7, 0.8),
    }
}

/// Unoptimized drawing of a grid with dynamic colormap.
pub fn draw_grid<T, F>(grid: &Array2<T>, to_color: F)
where
    F: Fn(&T) -> Color,
{
    for ((x, y), value) in grid.indexed_iter() {
        draw_rectangle(x as f32, y as f32, 1.0, 1.0, to_color(value));
    }
}

/// Drawing of a boolean grid. Only draw cells with true values. Useful for debugging.
pub fn draw_bool_grid(grid: &Array2<bool>, color: &Color, outline: &bool) {
    for ((x, y), value) in grid.indexed_iter() {
        if *value {
            if *outline {
                draw_rectangle_lines(x as f32, y as f32, 1.0, 1.0, 0.1, *color);
            } else {
                draw_rectangle(x as f32, y as f32, 1.0, 1.0, *color);
            }
        }
    }
}

/// Optimized variant of draw_grid using chunking. If a chunk has not been edited after
/// initialization, the entire chunk is drawn using a single rectangle. Otherwise, each block is
/// drawn individually as in the unoptimized variant.
pub fn draw_chunked_grid(
    grid: &Array2<TileTag>,
    chunks_edited: &Array2<bool>,
    chunk_size: usize,
) {
    for ((x_chunk, y_chunk), chunk_edited) in chunks_edited.indexed_iter() {
        if *chunk_edited {
            let x_start = x_chunk * chunk_size;
            let y_start = y_chunk * chunk_size;
            let x_end = usize::min((x_chunk + 1) * chunk_size, grid.shape()[0]);
            let y_end = usize::min((y_chunk + 1) * chunk_size, grid.shape()[1]);

            for x in x_start..x_end {
                for y in y_start..y_end {
                    let value = &grid[[x, y]];
                    draw_rectangle(x as f32, y as f32, 1.0, 1.0, blocktype_to_color(value));
                }
            }
        } else {
            let mut color = blocktype_to_color(&TileTag::Hookable); // assumed that initial value is hookable
            color.a = 0.95;
            draw_rectangle(
                (x_chunk * chunk_size) as f32,
                (y_chunk * chunk_size) as f32,
                chunk_size as f32,
                chunk_size as f32,
                color,
            );
        }
    }
}

pub fn draw_walker(walker: &Walker) {
    draw_rectangle_lines(
        walker.current_pos.x as f32,
        walker.current_pos.y as f32,
        1.0,
        1.0,
        2.0,
        colors::YELLOW,
    );
    draw_circle(
        walker.current_pos.x as f32 + 0.5,
        walker.current_pos.y as f32 + 0.5,
        0.25,
        colors::BLUE,
    )
}

pub fn draw_walker_kernel(walker: &Walker, kernel_type: KernelType) {
    let kernel = match kernel_type {
        KernelType::Inner => &walker.inner_kernel,
        KernelType::Outer => &walker.outer_kernel,
    };
    let offset: usize = kernel.size / 2; // offset of kernel wrt. position (top/left)

    let root_x = walker.current_pos.x.checked_sub(offset);
    let root_y = walker.current_pos.y.checked_sub(offset);

    if root_x.is_none() || root_y.is_none() {
        return; // dont draw as the following draw operation would fail
                // TODO: do this for each cell individually!
    }

    for ((x, y), kernel_active) in kernel.vector.indexed_iter() {
        if *kernel_active {
            draw_rectangle(
                (root_x.unwrap() + x) as f32,
                (root_y.unwrap() + y) as f32,
                1.0,
                1.0,
                match kernel_type {
                    KernelType::Inner => Color::new(0.0, 0.0, 1.0, 0.1),
                    KernelType::Outer => Color::new(0.0, 1.0, 0.0, 0.1),
                },
            );
        }
    }
}

pub fn draw_waypoints(waypoints: &[Vector2]) {
    for pos in waypoints.iter() {
        draw_circle(pos.x as f32 + 0.5, pos.y as f32 + 0.5, 1.0, colors::RED)
    }
}
