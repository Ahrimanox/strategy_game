extern crate piston;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;

use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL, TextureSettings, Filter, GlyphCache};
use piston::event_loop::{EventSettings, Events};
use piston::window::WindowSettings;

mod utils;
mod distance;
mod constraint;
mod path_planning;
mod map;
mod player;
mod game;

use game::Game;

fn main_game() {
    // Define OpenGL version we use
    let opengl = OpenGL::V4_5;

    // Create an Glutin window.
    let factor = 3.0 / 4.0;
    let window_width = 1600.0 * factor;
    let window_height = 900.0 * factor;
    let mut window: GlutinWindow = WindowSettings::new("Territory War", [window_width, window_height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .samples(16)
        .build()
        .unwrap();

    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let glyphs: GlyphCache = GlyphCache::new("fonts/FSEX300.ttf", (), texture_settings).expect("Failed to load font !");

    // Create a new game instance and run it.
    let map_size_level = 0;
    let mut game = Game {
        gl: Some(GlGraphics::new(opengl)),
        glyphs: Some(glyphs),

        background_color: [0.4, 0.0, 0.0, 1.0],

        grid_line_color: [0.0, 0.0, 0.0, 1.0],
        reachable_cell_color_mask: [1.0, 0.0, 0.0, 0.3],

        view_in_window_x: 32.0,
        view_in_window_y: 32.0,

        view_in_window_width: 800.0 * factor,
        view_in_window_height: 800.0 * factor,

        map_size_level: map_size_level,

        view_in_map_i: 0.0,
        view_in_map_j: 0.0,

        view_in_map_width: (2 as u32).pow(4+map_size_level) as f64+1.0,
        view_in_map_height: (2 as u32).pow(4+map_size_level) as f64+1.0,

        unit_default_speed: 10.0,
        player_num: 2,

        color_ramp_value: vec![0.0, 1.0],
        color_ramp_color: vec![[0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]],

        ..Game::default()
    };

    // Initialize game
    game.init();

    // Events processing loop
    let mut events: Events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        game.process_event(e);
    }
}

fn main() {
    // Launch game
    main_game();
}
