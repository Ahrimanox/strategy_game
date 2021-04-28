extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;

use std::collections::*;
use rand::prelude::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, Event, MouseCursorEvent, PressEvent, Button, MouseButton, ReleaseEvent, MouseScrollEvent};
use piston::window::WindowSettings;
use piston::input::keyboard::Key;

use graphics::*;
use graphics::rectangle::rectangle_by_corners;


#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Unit {
    // Team unit
    player: u8,

    // Unit position in grid
    position: [usize; 2],

    // Attributes
    damage: i32,
    health: i32,
    speed: i32,

    remaining_moves: i32
}


#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Building {
    // Team unit
    player: u8,

    // Unit position in grid
    position: [usize; 2],

    // Attributes
    damage: i32,
    health: i32,
}

fn diamond_square(n: u32, normalize: bool) -> Vec<Vec<f64>> {
    let base: usize = 2;
    let map_dim = base.pow(4+n)+1;

    let mut height_map: Vec<Vec<f64>> = Vec::with_capacity(map_dim);
    for i in 0..(map_dim) {
        height_map.push(Vec::with_capacity(map_dim));
        for _ in 0..(map_dim) {
            height_map[i].push(0.0);
        }
    }

    // Initialisation des coins
    height_map[0][0] = rand::thread_rng().gen_range((-(map_dim as f64))..=(map_dim as f64));
    height_map[0][map_dim-1] = rand::thread_rng().gen_range((-(map_dim as f64))..=(map_dim as f64));
    height_map[map_dim-1][0] = rand::thread_rng().gen_range((-(map_dim as f64))..=(map_dim as f64));
    height_map[map_dim-1][map_dim-1] = rand::thread_rng().gen_range((-(map_dim as f64))..=(map_dim as f64));

    let mut diamond_square_queue: VecDeque<(i32, i32, i32)> = VecDeque::new();
    diamond_square_queue.push_back((0, 0, map_dim as i32 - 1));
    let end_step = 2;
    while let Some((i, j, s)) = diamond_square_queue.pop_front() {
        // Diamant
        let rand_add = rand::thread_rng().gen_range((-(s as f64))..=(s as f64));
        let mean = (height_map[i as usize][j as usize] + height_map[i as usize][(j+s) as usize] + height_map[(i+s) as usize][j as usize] + height_map[(i+s) as usize][(j+s) as usize]) / 4.0;
        height_map[(i+s/2) as usize][(j+s/2) as usize] = mean + rand_add;

        // Carré
        let offsets_to_find_square_points = [(0, s/2), (s/2, 0), (s, s/2), (s/2, s)];
        let offsets_to_find_diamond_points = [(-s/2, 0), (0, -s/2), (s/2, 0), (0, s/2)];
        for (opi, opj) in offsets_to_find_square_points.iter() {
            let (pi, pj) = (i + opi, j + opj);

            let mut value_num: i32 = 0;
            let mut value_sum: f64 = 0.0;
            for (odpi, odpj) in offsets_to_find_diamond_points.iter() {
                let (dpi, dpj) = (pi + odpi, pj + odpj);
                if dpi >= 0 && dpi < map_dim as i32 && dpj >= 0 && dpj < map_dim as i32 {
                    value_num += 1;
                    value_sum += height_map[dpi as usize][dpj as usize];
                }
            }

            let rand_add = rand::thread_rng().gen_range((-(s as f64))..=(s as f64));
            height_map[pi as usize][pj as usize] = ((value_sum as f64) / (value_num as f64)) + rand_add;
        }

        // Population de la file d'attente
        if s > end_step {
            diamond_square_queue.push_back((i, j, s/2));
            diamond_square_queue.push_back((i, j+s/2, s/2));
            diamond_square_queue.push_back((i+s/2, j, s/2));
            diamond_square_queue.push_back((i+s/2, j+s/2, s/2));
        }
    }

    // Normalisation de la carte de hauteur
    if normalize {
        let mut min = 10.0 * map_dim as f64;
        let mut max = -10.0 * map_dim as f64;
        for h in height_map.iter().flat_map(|r| r.iter()) {
            if *h < min {min = *h;}
            if *h > max {max = *h;}
        }

        println!("Min = {}, Max = {}", min, max);

        for i in 0..map_dim {
            for j in 0..map_dim {
                height_map[i][j] = (height_map[i][j] - min) / (max - min);
            }
        }

        let mut min = 3.0 * map_dim as f64;
        let mut max = -3.0 * map_dim as f64;
        for h in height_map.iter().flat_map(|r| r.iter()) {
            if *h < min {min = *h;}
            if *h > max {max = *h;}
        }
    }

    height_map
}

#[derive(Default)]
pub struct Game {

    // OpenGL drawing backend
    gl: Option<GlGraphics>,

    // Defined colors
    background_color: [f32; 4], // Color of the background
    grid_line_color: [f32; 4],
    players_color: [[f32; 4]; 2],
    reachable_cell_color_mask: [f32; 4],

    view_in_window_x: f64,
    view_in_window_y: f64,

    view_in_window_width: f64,
    view_in_window_height: f64,

    map_size_level: u32,
    map_size: usize,

    view_in_map_i: f64,
    view_in_map_j: f64,

    view_in_map_width: f64,
    view_in_map_height: f64,

    // Game variables
    unit_default_speed: i32,

    // Unit map (hold Unit for each position in the map
    unit_map: Vec<Vec<Option<Unit>>>,

    // Height map
    height_map: Vec<Vec<f64>>,

    color_ramp_value: Vec<f64>,
    color_ramp_color: Vec<[f32; 4]>,

    active_player: u8,
    active_unit_position: Option<[usize; 2]>,
    last_mouse_position: Option<[f64; 2]>,

    pressed_grid_cell: Option<[usize; 2]>,
    released_grid_cell: Option<[usize; 2]>,
}

impl Game {
    fn init(&mut self) {
        self.map_size = (2 as usize).pow(4+self.map_size_level)+1;

        // Initialize unit map with None
        self.unit_map = Vec::with_capacity(self.map_size);
        for i in 0..(self.map_size) {
            self.unit_map.push(Vec::with_capacity(self.map_size));
            for _ in 0..(self.map_size) {
                self.unit_map[i].push(None);
            }
        }

        // Initialize height map with random height value [0, 1]
        let mut rng = rand::thread_rng();
        self.height_map = Vec::with_capacity(self.map_size);
        for i in 0..(self.map_size) {
            self.height_map.push(Vec::with_capacity(self.map_size));
            for _ in 0..(self.map_size) {
                self.height_map[i].push(rng.gen());
            }
        }
        self.height_map = diamond_square(self.map_size_level, true);

        // Let the first player be the current active player
        self.active_player = 0;

        // Add first units
        let mut units_to_add = Vec::new();
        units_to_add.push(Unit {
            player: 0,
            position: [0, 0],
            damage: 1,
            health: 1,
            speed: self.unit_default_speed,
            remaining_moves: self.unit_default_speed
        });
        units_to_add.push(Unit {
            player: 0,
            position: [3, 3],
            damage: 1,
            health: 1,
            speed: self.unit_default_speed,
            remaining_moves: self.unit_default_speed
        });
        units_to_add.push(Unit {
            player: 1,
            position: [self.map_size - 1, self.map_size - 1],
            damage: 1,
            health: 1,
            speed: self.unit_default_speed,
            remaining_moves: self.unit_default_speed
        });

        // Add units to unit map 
        for unit in units_to_add.drain(..) {
            self.unit_map[unit.position[0]][unit.position[1]] = Some(unit);
        }
    }

    // Utility functions
    fn cell_pixel(&self) -> (f64, f64) {
        (self.view_in_window_width / self.view_in_map_width, self.view_in_window_height / self.view_in_map_height)
    }

    fn visible_map_bounds(&self) -> (i32, i32, i32, i32) {
        (std::cmp::max(self.view_in_map_i.floor() as i32, 0), std::cmp::min((self.view_in_map_i + self.view_in_map_height).ceil() as i32, self.map_size as i32), 
        std::cmp::max(self.view_in_map_j.floor() as i32, 0), std::cmp::min((self.view_in_map_j + self.view_in_map_width).ceil() as i32, self.map_size as i32))
    }

    fn i_to_y(&self, i: i32) -> f64 {
        ((i as f64 - self.view_in_map_i) / self.view_in_map_height) * self.view_in_window_height + self.view_in_window_y
    }
    
    fn j_to_x(&self, j: i32) -> f64 {
        ((j as f64 - self.view_in_map_j) / self.view_in_map_width) * self.view_in_window_width + self.view_in_window_x
    }

    fn h_to_color(&self, h: f64, interpolate: bool) -> [f32; 4] {
        if h < self.color_ramp_value[0] {
            return self.color_ramp_color[0];
        }

        if h >= self.color_ramp_value[self.color_ramp_color.len() - 1] {
            return self.color_ramp_color[self.color_ramp_color.len() - 1];
        }

        if interpolate {
            for i in 0..(self.color_ramp_value.len() - 1) {
                let lvalue = self.color_ramp_value[i];
                let rvalue = self.color_ramp_value[i+1];
                let lcolor = self.color_ramp_color[i];
                let rcolor = self.color_ramp_color[i+1];
                if h >= lvalue && h < rvalue {
                    let alpha: f32 = ((h - lvalue) / (rvalue - lvalue)) as f32;
                    return [lcolor[0] * alpha + rcolor[0] * (1.0 - alpha), 
                            lcolor[1] * alpha + rcolor[1] * (1.0 - alpha), 
                            lcolor[2] * alpha + rcolor[2] * (1.0 - alpha), 
                            lcolor[3] * alpha + rcolor[3] * (1.0 - alpha)];
                }
            }
        }
        else {
            for i in 0..(self.color_ramp_value.len() - 1) {
                if h >= self.color_ramp_value[i] && h < self.color_ramp_value[i+1] {
                    return self.color_ramp_color[i];
                }
            }   
        }

        return [0., 0., 0., 0.];
    }

    // Event and Update methods
    fn process_event(&mut self, event: Event) {
        // Get the latest mouse position
        if let Some(args) = event.mouse_cursor_args() {
            self.last_mouse_position = Some(args);
        }

        // Mouse button pressed
        if let Some(Button::Mouse(mouse_button)) = event.press_args() {
            // Left mouse button pressed
            if mouse_button == MouseButton::Left {
                let last_mouse_pos = self.last_mouse_position.unwrap();

                // TODO : Make a function with that
                let (mx, my) = (last_mouse_pos[0], last_mouse_pos[1]);
                let mi = (((my - self.view_in_window_y) / self.view_in_window_height) * self.view_in_map_height as f64 + self.view_in_map_i as f64) as i32;
                let mj = (((mx - self.view_in_window_x) / self.view_in_window_width) * self.view_in_map_width as f64 + self.view_in_map_j as f64) as i32;

                if mi >= 0 && mi < self.map_size as i32 && mj >=0 && mj < self.map_size as i32 {
                    self.pressed_grid_cell = Some([mi as usize, mj as usize]);
                }
                else {
                    self.pressed_grid_cell = None;
                }
            }

            // Right mouse button pressed
            if mouse_button == MouseButton::Right {
                // DO Turn Rollover (Reset moves of other player unit) -- TODO Make a function turn_rollover
                for unit in self.unit_map.iter_mut().flat_map(|unit_row| unit_row.iter_mut()) {
                    if let Some(unit) = unit {
                        unit.remaining_moves = unit.speed;
                    }
                }

                // Change active player and reset active unit position
                self.active_player = 1 - self.active_player;
                self.active_unit_position = None;
            }
        }

        // Mouse button released
        if let Some(Button::Mouse(mouse_button)) = event.release_args() {
            // Left mouse button released
            if mouse_button == MouseButton::Left {
                let last_mouse_pos = self.last_mouse_position.unwrap();

                // TODO : Make a function with that
                let (mx, my) = (last_mouse_pos[0], last_mouse_pos[1]);
                let mi = (((my - self.view_in_window_y) / self.view_in_window_height) * self.view_in_map_height as f64 + self.view_in_map_i as f64) as i32;
                let mj = (((mx - self.view_in_window_x) / self.view_in_window_width) * self.view_in_map_width as f64 + self.view_in_map_j as f64) as i32;

                if mi >= 0 && mi < self.map_size as i32 && mj >=0 && mj < self.map_size as i32 {
                    self.released_grid_cell = Some([mi as usize, mj as usize]);
                }
                else {
                    self.released_grid_cell = None;
                }

                // Click event
                if self.pressed_grid_cell == self.released_grid_cell {
                    if let Some(released_grid_cell) = self.released_grid_cell {
                        println!("Click on i={}, j={} ...", released_grid_cell[0], released_grid_cell[1]);

                        // Check if there is an active unit
                        if let Some(active_unit_position) = self.active_unit_position {
                            if let Some(active_unit) = &self.unit_map[active_unit_position[0]][active_unit_position[1]] {
                                println!("... while having active unit : {:?} ...", active_unit);
                                if let Some(underlying_unit) = &self.unit_map[released_grid_cell[0]][released_grid_cell[1]] {
                                    println!("... and clicking on unit : {:?}", underlying_unit);
                                    // Same unit --> Deactivation of unit
                                    if underlying_unit == active_unit {
                                        self.active_unit_position = None;
                                        println!("Deactivation of unit : {:?}", underlying_unit);
                                    }
                                    // Unit of other player --> Possible attacks
                                    else if active_unit.player != underlying_unit.player {
                                        println!("Possible attacks of current active unit {:?} against {:?}", active_unit, underlying_unit);
                                    }
                                    // Another unit of the same player --> Deactivation and Activation of other unit
                                    else {
                                        println!("Deactivation of current active unit {:?} and activation of {:?}", active_unit, underlying_unit);
                                        self.active_unit_position = Some(underlying_unit.position);
                                    }
                                }
    
                                // No underlying unit and active unit --> Possible moves
                                else {
                                    let d = (released_grid_cell[0] as i32 - active_unit_position[0] as i32).abs() + (released_grid_cell[1] as i32 - active_unit_position[1] as i32).abs();
                                    if d <= active_unit.remaining_moves as i32 {
                                        // Make the moves
                                        self.unit_map[released_grid_cell[0]][released_grid_cell[1]] = self.unit_map[active_unit_position[0]][active_unit_position[1]];
                                        self.unit_map[active_unit_position[0]][active_unit_position[1]] = None;

                                        // Update active unit remaining moves and position
                                        if let Some(active_unit) = &mut self.unit_map[released_grid_cell[0]][released_grid_cell[1]] {
                                            active_unit.remaining_moves -= d;
                                            active_unit.position = released_grid_cell;
                                        }

                                        // Update active unit positition
                                        self.active_unit_position = Some(released_grid_cell);
                                    }
                                }
                            }
                        }

                        // No active unit 
                        else {
                            // Activation underlying unit
                            if let Some(underlying_unit) = &self.unit_map[released_grid_cell[0]][released_grid_cell[1]] {
                                println!("Click on unit : {:?}", underlying_unit);
                                if underlying_unit.player == self.active_player {
                                    self.active_unit_position = Some(underlying_unit.position);
                                }
                            }
                        }
                    }   
                }

                self.pressed_grid_cell = None;
            }
        }

        if let Some(Button::Keyboard(key)) = event.press_args() {
            let view_move: f64 = 10.0;
            let n: f64 = 2.0;
            match key {
                Key::Up => {
                    self.view_in_map_i = (self.view_in_map_i - view_move).max(-self.view_in_map_height / n);
                },
                Key::Down => {
                    self.view_in_map_i = (self.view_in_map_i + view_move).min(self.map_size as f64 - (1.0 - (1.0 / n)) * self.view_in_map_height);
                },
                Key::Left => {
                    self.view_in_map_j = (self.view_in_map_j - view_move).max(-self.view_in_map_width / n);
                },
                Key::Right => {
                    self.view_in_map_j = (self.view_in_map_j + view_move).min(self.map_size as f64 - (1.0 - (1.0 / n)) * self.view_in_map_height);
                },
                _ => {}
            }
        }

        // Mouse scroll event
        if let Some(args) = event.mouse_scroll_args() {
            let (_scroll_x, scroll_y) = (args[0], args[1]);

            let (view_center_in_map_i, view_center_in_map_j) = (
                self.view_in_map_i + self.view_in_map_height / 2.0, 
                self.view_in_map_j + self.view_in_map_width / 2.0
            );

            if scroll_y == 1.0 {
                let new_view_in_map_width = self.view_in_map_width / 2.0;
                let new_view_in_map_height = self.view_in_map_height / 2.0;
                if new_view_in_map_width > 3.0 && new_view_in_map_height > 3.0 {
                    self.view_in_map_width = new_view_in_map_width;
                    self.view_in_map_height = new_view_in_map_height;
                    self.view_in_map_i = view_center_in_map_i - self.view_in_map_height / 2.0;
                    self.view_in_map_j = view_center_in_map_j - self.view_in_map_width / 2.0;
                }
            }

            else if scroll_y == -1.0 {
                let new_view_in_map_width = self.view_in_map_width * 2.0;
                let new_view_in_map_height = self.view_in_map_height * 2.0;
                self.view_in_map_width = new_view_in_map_width;
                self.view_in_map_height = new_view_in_map_height;
                self.view_in_map_i = view_center_in_map_i - self.view_in_map_height / 2.0;
                self.view_in_map_j = view_center_in_map_j - self.view_in_map_width / 2.0;
            }
        }

        if let Some(args) = event.render_args() {
            self.render(&args);
        }
    }

    // Render methods
    fn render_grid(&mut self, c: Context, draw_lines: bool) {

        // Draw grid cells
        let (cell_pix_width, cell_pix_height) = self.cell_pixel();
        let cell = rectangle_by_corners(0.0, 0.0, cell_pix_width, cell_pix_height);

        let (view_in_map_i1, view_in_map_i2, view_in_map_j1, view_in_map_j2) = self.visible_map_bounds();

        for i in view_in_map_i1..view_in_map_i2 {
            let y = self.i_to_y(i);
            for j in view_in_map_j1..view_in_map_j2 {
                let x = self.j_to_x(j);
                let transform = c.transform.trans(x, y);

                // Draw each grid cell
                rectangle(self.h_to_color(self.height_map[i as usize][j as usize], false), cell, transform, self.gl.as_mut().unwrap());
            }
        }

        // Draw pressed grid cells
        if let Some(pressed_grid_cell) = self.pressed_grid_cell {
            let x = self.j_to_x(pressed_grid_cell[1] as i32);
            let y = self.i_to_y(pressed_grid_cell[0] as i32);
            let transform = c.transform.trans(x, y);
            rectangle([1.0, 0.0, 1.0, 1.0], cell, transform, self.gl.as_mut().unwrap());
        }

        // Draw grid lines
        let horizontal_line_thickness = (1.0 as f64).max(cell_pix_width / 100.0);
        let vertical_line_thickness = (1.0 as f64).max(cell_pix_height / 100.0);

        let horizontal_line =
            rectangle_by_corners(
                0.0, 0.0,
                self.j_to_x(view_in_map_j2) - self.j_to_x(view_in_map_j1), horizontal_line_thickness
            );

        let vertical_line =
            rectangle_by_corners(
                0.0, 0.0,
                vertical_line_thickness, self.i_to_y(view_in_map_i2) - self.i_to_y(view_in_map_i1)
            );
        
        if draw_lines {
            for i in view_in_map_i1..view_in_map_i2 {
                // Draw each horizontal line
                let y = self.i_to_y(i);
                rectangle(self.grid_line_color, horizontal_line, c.transform.trans(self.j_to_x(view_in_map_j1), y), self.gl.as_mut().unwrap());
            }
    
            for j in view_in_map_j1..view_in_map_j2 {
                // Draw each horizontal line
                let x = self.j_to_x(j);
                rectangle(self.grid_line_color, vertical_line, c.transform.trans(x, self.i_to_y(view_in_map_i1)), self.gl.as_mut().unwrap());
            }
        }
    }

    fn render_unit_reachable_cells(&mut self, c: Context) {
        // Draw reachable mask at reachable cells by active unit if there is an active one
        if let Some(active_unit_position) = self.active_unit_position {
            if let Some(active_unit) = &self.unit_map[active_unit_position[0]][active_unit_position[1]] {
                // Compute cell dimensions in pixel and define reachable mask shape
                let (cell_pix_width, cell_pix_height) = self.cell_pixel();
                let reachable_cell = rectangle_by_corners(0.0, 0.0, cell_pix_width, cell_pix_height);
                let (view_in_map_i1, view_in_map_i2, view_in_map_j1, view_in_map_j2) = self.visible_map_bounds();

                // Draw each reachable cell by active unit
                let (pi, pj) = (active_unit_position[0] as i32, active_unit_position[1] as i32);
                let (ibeg, iend) = (std::cmp::max(pi - active_unit.remaining_moves, view_in_map_i1), std::cmp::min(pi + active_unit.remaining_moves, view_in_map_i2) + 1);
                let (jbeg, jend) = (std::cmp::max(pj - active_unit.remaining_moves, view_in_map_j1), std::cmp::min(pj + active_unit.remaining_moves, view_in_map_j2) + 1);
                for i in ibeg..iend {
                    for j in jbeg..jend {
                        let d = (pi - i).abs() + (pj - j).abs();
                        if d <= active_unit.remaining_moves && d > 0 {
                            let transform = c.transform.trans(self.j_to_x(j), self.i_to_y(i));
                            rectangle(self.reachable_cell_color_mask, reachable_cell, transform, self.gl.as_mut().unwrap());
                        }
                    }
                }
            }
        }
    }

    fn render_unit(&mut self, c: Context) {
        // Compute cell dimensions in pixel and define reachable mask shape
        let (cell_pix_width, cell_pix_height) = self.cell_pixel();
        let (view_in_map_i1, view_in_map_i2, view_in_map_j1, view_in_map_j2) = self.visible_map_bounds();

        let unit_shape =
            rectangle_by_corners(
                cell_pix_width / 16.0, cell_pix_height / 16.0,
                cell_pix_width - cell_pix_width / 16.0 + 1.0,
                cell_pix_height - cell_pix_height / 16.0 + 1.0
            );

        // Draw units
        for i in view_in_map_i1..view_in_map_i2 {
            for j in view_in_map_j1..view_in_map_j2 {
                if let Some(unit) = &self.unit_map[i as usize][j as usize] {
                    ellipse(
                        self.players_color[unit.player as usize], unit_shape, 
                        c.transform.trans(self.j_to_x(j), self.i_to_y(i)), self.gl.as_mut().unwrap()
                    );
                }
            }
        }
    }

    fn render(&mut self, args: &RenderArgs) {
        // Background color
        let background_color = self.background_color;

        // Get OpenGL context and begin the drawing pipeline
        let c: Context = self.gl.as_mut().unwrap().draw_begin(args.viewport());

        // Clear the background
        clear(background_color, self.gl.as_mut().unwrap());

        // Render grid
        self.render_grid(c, false);

        // Render reachable cell by active unit
        self.render_unit_reachable_cells(c);

        // Render units
        self.render_unit(c);

        // End the drawing pipeline
        self.gl.as_mut().unwrap().draw_end();
    }
}


fn main_game() {
    // Define OpenGL version we use
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

    // Create a new game instance and run it.
    let map_size_level = 4;
    let deep_water = 0.5;
    let soft_water = 0.05;
    let sand = 0.025;
    let grass = 0.2475;
    let mut game = Game {
        gl: Some(GlGraphics::new(opengl)),

        background_color: [0.0, 0.0, 0.0, 1.0],

        grid_line_color: [0.0, 0.0, 0.0, 1.0],
        players_color: [[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]],
        reachable_cell_color_mask: [1.0, 0.0, 0.0, 0.15],

        view_in_window_x: 0.0,
        view_in_window_y: 0.0,

        view_in_window_width: 800.0,
        view_in_window_height: 800.0,

        map_size_level: map_size_level,

        view_in_map_i: 0.0,
        view_in_map_j: 0.0,

        view_in_map_width: (2 as u32).pow(4+map_size_level) as f64+1.0,
        view_in_map_height: (2 as u32).pow(4+map_size_level) as f64+1.0,

        unit_default_speed: 3,

        active_unit_position: None,

        color_ramp_value: vec![
            0.0, // Eau profonde
            0.25, // Eau douce
            0.30, // Sable
            0.35, // Herbe
            0.85, // Montagne
            0.95 // Neige
        ],
        // color_ramp_value: vec![0.0, 0.45, 0.75, 0.9625, 0.99],
        // color_ramp_value: vec![0.0, 1.0],
        color_ramp_color: vec![[0.007, 0.176, 0.357, 1.0], // Eau profonde
                               [0.051, 0.286, 0.404, 1.0], // Eau douce
                               [0.051, 0.286, 0.404, 1.0], // Sable
                               [0.204, 0.412, 0.180, 1.0], // Herbe
                               [0.557, 0.541, 0.341, 1.0], // Montage
                               [1.0, 1.0, 1.0, 1.0]], // Neige
        // color_ramp_color: vec![[0.0, 0.0, 0.0, 1.0], [1.0, 1.0, 1.0, 1.0]],

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
    main_game();
}
