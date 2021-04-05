extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs};
use piston::window::WindowSettings;

use graphics::*;
use graphics::rectangle::rectangle_by_corners;
use piston::{MouseCursorEvent, Event, PressEvent, Button, MouseButton, ReleaseEvent};

pub struct Unit {
    
    // Team unit
    player: u8,

    // Unit position in grid
    grid_pos_x: usize,
    grid_pos_y: usize,

    // Attributes
    moves: usize
}

#[derive(Default)]
pub struct Game {

    // OpenGL drawing backend
    gl: Option<GlGraphics>,

    // Render variables
    window_height: f64,
    window_width: f64,

    // Defined colors
    background_color: [f32; 4], // Color of the background
    grid_cell_color: [f32; 4],
    grid_line_color: [f32; 4],
    players_color: [[f32; 4]; 2],

    // Grid variables
    cell_num_along_w: usize,
    cell_num_along_h: usize,

    // Game variables
    moves_per_unit: usize,

    // Unit map (hold Unit for each position in the map
    unit_map: Vec<Option<Unit>>,

    active_player: u8,
    active_unit_pos: Option<[usize; 2]>,
    last_mouse_pos: Option<[f64; 2]>,
    last_pressed_mouse_pos: Option<[f64; 2]>,

    pressed_grid_cell: Option<[usize; 2]>,

    // last_render_args: Option<RenderArgs>,
}

impl Game {
    fn init(&mut self) {
        // Initialize unit map and add first units
        let position_num = self.cell_num_along_w * self.cell_num_along_h;
        for _c in 0..(position_num) {
            self.unit_map.push(None);
        }

        // Let the first player be the current active player
        self.active_player = 0;

        let red_unit = Some(Unit {
            player: 0,
            grid_pos_x: 0,
            grid_pos_y: 0,

            moves: self.moves_per_unit
        });
        let blue_unit = Some(Unit {
            player: 1,
            grid_pos_x: self.cell_num_along_w - 1,
            grid_pos_y: self.cell_num_along_h - 1,

            moves: self.moves_per_unit
        });
        self.unit_map[0] = red_unit;
        self.unit_map[(self.cell_num_along_w * self.cell_num_along_h) - 1] = blue_unit;
    }

    // fn get_underlying_unit(self, mouse_position: [usi])

    // Event and Update methods
    fn process_event(&mut self, event: Event) {
        // Get the latest mouse position
        if let Some(args) = event.mouse_cursor_args() {
            self.last_mouse_pos = Some(args);
            println!("{:?}", args);
        }

        // Mouse button pressed
        if let Some(Button::Mouse(mouse_button)) = event.press_args() {
            if mouse_button == MouseButton::Left {
                let last_mouse_pos = self.last_mouse_pos.unwrap();
                let cell_pix_width: f64 = self.window_width / self.cell_num_along_w as f64;
                let cell_pix_height: f64 = self.window_height / self.cell_num_along_h as f64;
                let pressed_grid_cell = [
                    (last_mouse_pos[1] / cell_pix_height) as usize,
                    (last_mouse_pos[0] / cell_pix_width) as usize
                ];

                self.pressed_grid_cell = Some(pressed_grid_cell);
            }
        }

        // Mouse button released
        if let Some(Button::Mouse(mouse_button)) = event.release_args() {
            if mouse_button == MouseButton::Left {

                // Check if released cell is the same as pressed one --> 'click-event'
                let last_mouse_pos = self.last_mouse_pos.unwrap();
                let cell_pix_width: f64 = self.window_width / self.cell_num_along_w as f64;
                let cell_pix_height: f64 = self.window_height / self.cell_num_along_h as f64;
                let released_grid_cell = [
                    (last_mouse_pos[1] / cell_pix_height) as usize,
                    (last_mouse_pos[0] / cell_pix_width) as usize
                ];

                if let Some(pressed_grid_cell) = self.pressed_grid_cell {
                    if pressed_grid_cell == released_grid_cell {
                        // Active underlying unit if it is unit of current player
                        // And there is not another active unit
                        let underlying_idx = released_grid_cell[1] * self.cell_num_along_w + released_grid_cell[0];
                        if let Some(underlying_unit) = &self.unit_map[underlying_idx] {
                            // Existing or Non-Existing active unit
                            if let Some(active_unit_pos) = self.active_unit_pos {

                            }
                            else {
                                if underlying_unit.player == self.active_player {
                                    self.active_unit_pos = Some(released_grid_cell);
                                }
                            }
                        }
                    }
                    else {}
                }

                // Get active unit and its position


                self.pressed_grid_cell = None;
            }
        }

        if let Some(args) = event.render_args() {
            self.render(&args);
        }

    }

    fn update(&mut self, args: &UpdateArgs) {

    }

    // Render methods
    fn render_grid(&mut self, c: Context, window_width: f64, window_height: f64) {

        let gl = self.gl.as_mut().unwrap();

        // First, draw grid cells
        let cell_pix_width: f64 = window_width / self.cell_num_along_w as f64;
        let cell_pix_height: f64 = window_height / self.cell_num_along_h as f64;
        let cell =
            rectangle_by_corners(
                0.0, 0.0,
                cell_pix_width, cell_pix_height
            );

        for i in 0..self.cell_num_along_h {
            for j in 0..self.cell_num_along_w {
                let transform = c.
                    transform.
                    trans(j as f64 * cell_pix_width,
                          i as f64 * cell_pix_height);

                // Draw each cell
                rectangle(self.grid_cell_color,
                          cell,
                          transform,
                          gl);
            }
        }

        // Draw pressed grid cells
        if let Some(pressed_grid_cell) = self.pressed_grid_cell {
            let transform = c.
                    transform.
                    trans(pressed_grid_cell[1] as f64 * cell_pix_width,
                          pressed_grid_cell[0] as f64 * cell_pix_height);

                // Draw each cell
                rectangle([1.0, 0.0, 1.0, 1.0],
                          cell,
                          transform,
                          gl);
        }

        // Draw grid lines
        let horizontal_line_thickness = 1.0;
        let vertical_line_thickness = 1.0;

        let horizontal_line =
            rectangle_by_corners(
                0.0, 0.0,
                window_width, horizontal_line_thickness
            );

        let vertical_line =
            rectangle_by_corners(
                0.0, 0.0,
                vertical_line_thickness, window_height
            );
        for i in 0..self.cell_num_along_h {
            // Draw each horizontal line
            rectangle(self.grid_line_color,
                      horizontal_line,
                      c.
                          transform.
                          trans(0.0, i as f64 * cell_pix_height),
                      gl);

            rectangle(self.grid_line_color,
                      horizontal_line,
                      c.
                          transform.
                          trans(0.0, (i+1) as f64 * cell_pix_height - horizontal_line_thickness),
                      gl);
        }

        for j in 0..self.cell_num_along_w {
            // Draw each vertical line
            rectangle(self.grid_line_color,
                      vertical_line,
                      c.
                          transform.
                          trans(j as f64 * cell_pix_width, 0.0),
                      gl);

            rectangle(self.grid_line_color,
                      vertical_line,
                      c.
                          transform.
                          trans((j+1) as f64 * cell_pix_width - vertical_line_thickness, 0.0),
                      gl);
        }
    }

    fn render_unit(&mut self, c: Context, window_width: f64, window_height: f64) {

        // Define unit shape and cell pixel dimensions
        let cell_pix_width: f64 = window_width / self.cell_num_along_w as f64;
        let cell_pix_height: f64 = window_height / self.cell_num_along_h as f64;

        let unit_shape =
            rectangle_by_corners(
                cell_pix_width / 20.0, cell_pix_height / 20.0,
                cell_pix_width - cell_pix_width / 20.0,
                cell_pix_height - cell_pix_height / 20.0
            );

        // Draw units
        for (idx, unit_opt) in self.unit_map.iter().enumerate() {
            match unit_opt {
                Some(unit) => {
                    let unit_pos_x = idx % self.cell_num_along_w;
                    let unit_pos_y = idx / self.cell_num_along_h;

                    ellipse(self.players_color[unit.player as usize],
                            unit_shape,
                            c.
                                transform.trans(unit_pos_x as f64 * cell_pix_width,
                                                unit_pos_y as f64 * cell_pix_height),
                            self.gl.as_mut().unwrap()
                    );
                },
                None => {}
            }
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        // Declare variable for friendly-usage
        let window_width = args.window_size[0];
        let window_height = args.window_size[1];

        let background_color= self.background_color;

        // Get OpenGL context and begin the drawing pipeline
        let c: Context = self.gl.as_mut().unwrap().draw_begin(args.viewport());

        // Clear the background
        clear(background_color, self.gl.as_mut().unwrap());

        // Render grid
        self.render_grid(c, window_width, window_height);

        // Render units
        self.render_unit(c, window_width, window_height);

        // End the drawing pipeline
        self.gl.as_mut().unwrap().draw_end();
    }
}

fn main_game() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V4_5;

    // Create an Glutin window.
    let window_width = 800.0;
    let window_height = 800.0;
    let mut window: Window = WindowSettings::new("strategy game", [window_width, window_height])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut game = Game {
        gl: Some(GlGraphics::new(opengl)),

        window_width,
        window_height,

        background_color: [0.0, 0.0, 0.0, 1.0],

        grid_cell_color: [0.0, 0.603, 0.09, 1.0],
        grid_line_color: [0.0, 0.0, 0.0, 1.0],
        players_color: [[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]],

        cell_num_along_w: 9,
        cell_num_along_h: 9,

        moves_per_unit: 2,

        active_unit_pos: None,

        ..Game::default()
    };

    game.init();

    let mut events: Events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        game.process_event(e);
    }
}


fn main() {
    main_game();
}
