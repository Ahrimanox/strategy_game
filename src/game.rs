use piston::input::{RenderArgs, RenderEvent, Event, MouseCursorEvent, PressEvent, Button, MouseButton, ReleaseEvent, MouseScrollEvent};
use piston::input::keyboard::Key;

use opengl_graphics::{GlGraphics, GlyphCache};

use graphics::*;
use graphics::rectangle::rectangle_by_corners;

use graphics::text::Text;
use graphics::rectangle::Rectangle;
use graphics::ellipse::Ellipse;

use rand::prelude::*;

use crate::map::{Map, diamond_square};
use crate::player::{Unit, Building, Player};


#[derive(Default)]
pub struct Game<'a> {

    // Rendering attributes
    pub gl: Option<GlGraphics>,
    pub glyphs: Option<GlyphCache<'a>>,

    pub background_color: [f32; 4],
    pub grid_line_color: [f32; 4],
    pub reachable_cell_color_mask: [f32; 4],

    pub view_in_window_x: f64,
    pub view_in_window_y: f64,

    pub view_in_window_width: f64,
    pub view_in_window_height: f64,

    pub map_size_level: u32,
    pub map_size: usize,

    pub view_in_map_i: f64,
    pub view_in_map_j: f64,

    pub view_in_map_width: f64,
    pub view_in_map_height: f64,

    // Game variables
    pub unit_default_speed: i32,
    pub player_num: usize,

    // Player vector
    pub players: Vec<Player>,

    // Building map (hold building for each position in the map
    pub building_map: Map<Option<Building>>,

    // Unit map (hold Unit for each position in the map
    pub unit_map: Map<Option<Unit>>,

    // Territory map
    pub territory_map: Map<usize>,

    // Height map
    pub height_map: Map<f64>,

    pub color_ramp_value: Vec<f64>,
    pub color_ramp_color: Vec<[f32; 4]>,

    pub active_player: usize,
    pub active_unit_position: Option<[usize; 2]>,
    pub latest_mouse_position: Option<[f64; 2]>,

    pub pressed_map_cell: Option<[usize; 2]>,
    pub released_map_cell: Option<[usize; 2]>,
}

impl Game<'_> {
    // Init game
    pub fn init(&mut self) {
        self.map_size = (2 as usize).pow(4+self.map_size_level)+1;

        // Initialize unit map and building map with None
        self.unit_map = Map::<Option<Unit>>::new(self.map_size, self.map_size, None);
        self.building_map = Map::<Option<Building>>::new(self.map_size, self.map_size, None);
        self.territory_map = Map::<usize>::new(self.map_size, self.map_size, self.player_num);

        // Initialize height map with random height value [0, 1]
        let mut rng = rand::thread_rng();
        self.height_map = Map::<f64>::new(self.map_size, self.map_size, 0.0);
        for i in 0..(self.map_size) {
            for j in 0..(self.map_size) {
                self.height_map[(i, j)] = rng.gen();
            }
        }
        self.height_map = diamond_square(self.map_size_level, true);

        // Initialize players 
        // TODO : Initialize base position for all players with clever algorithm
        self.players = Vec::<Player>::new();
        self.players.push(Player::new([0, 0], [1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]));
        self.players.push(Player::new([self.map_size - 1, self.map_size - 1], [0.0, 0.0, 1.0, 1.0], [0.0, 0.0, 0.0, 1.0]));

        // Let the first player be the current active player
        self.active_player = 0;

        // Add "base" buildings
        let mut buildings_to_add = Vec::new();
        buildings_to_add.push(Building {
            player: 0,
            position: [0, 0],
            damage: 0,
            health: 1
        });
        buildings_to_add.push(Building {
            player: 1,
            position: [self.map_size - 1, self.map_size - 1],
            damage: 0,
            health: 1
        });
        for building in buildings_to_add.drain(..) {
            self.building_map[(building.position[0], building.position[1])] = Some(building);
            self.territory_map[(building.position[0], building.position[1])] = building.player;
        }

        // Add first units
        let mut units_to_add = Vec::new();
        units_to_add.push(Unit {
            player: 0,
            position: [0, 1],
            damage: 1,
            health: 1,
            speed: self.unit_default_speed,
            remaining_moves: self.unit_default_speed
        });
        units_to_add.push(Unit {
            player: 0,
            position: [1, 0],
            damage: 1,
            health: 1,
            speed: self.unit_default_speed,
            remaining_moves: self.unit_default_speed
        });
        units_to_add.push(Unit {
            player: 1,
            position: [self.map_size - 1, self.map_size - 2],
            damage: 1,
            health: 1,
            speed: self.unit_default_speed,
            remaining_moves: self.unit_default_speed
        });
        units_to_add.push(Unit {
            player: 1,
            position: [self.map_size - 2, self.map_size - 1],
            damage: 1,
            health: 1,
            speed: self.unit_default_speed,
            remaining_moves: self.unit_default_speed
        });

        // Add units to unit map 
        for unit in units_to_add.drain(..) {
            self.unit_map[(unit.position[0], unit.position[1])] = Some(unit);
            self.territory_map[(unit.position[0], unit.position[1])] = unit.player;
        }

        // Look at active player base position
        let active_player_base_position = self.players[self.active_player].base_position;
        self.view_in_map_width = 32.0;
        self.view_in_map_height = 32.0;
        self.look_at([active_player_base_position[0] as f64, active_player_base_position[1] as f64]);
    }

    // Utility functions
    fn cell_pixel(&self) -> (f64, f64) {
        (self.view_in_window_width / self.view_in_map_width, self.view_in_window_height / self.view_in_map_height)
    }

    fn visible_map_bounds(&self) -> (i32, i32, i32, i32) {
        (std::cmp::max(self.view_in_map_i.floor() as i32, 0), std::cmp::max(self.view_in_map_j.floor() as i32, 0),
        std::cmp::min((self.view_in_map_i + self.view_in_map_height).ceil() as i32, self.map_size as i32), 
        std::cmp::min((self.view_in_map_j + self.view_in_map_width).ceil() as i32, self.map_size as i32))
    }

    fn i_to_y(&self, i: i32) -> f64 {
        ((i as f64 - self.view_in_map_i) / self.view_in_map_height) * self.view_in_window_height + self.view_in_window_y
    }
    
    fn j_to_x(&self, j: i32) -> f64 {
        ((j as f64 - self.view_in_map_j) / self.view_in_map_width) * self.view_in_window_width + self.view_in_window_x
    }

    fn window_position_to_map_position(&self, window_position: (f64, f64)) -> (i32, i32) {
        let (x, y) = window_position;
        let i = (((y - self.view_in_window_y) / self.view_in_window_height) * self.view_in_map_height as f64 + self.view_in_map_i as f64) as i32;
        let j = (((x - self.view_in_window_x) / self.view_in_window_width) * self.view_in_map_width as f64 + self.view_in_map_j as f64) as i32;
        (i, j)
    }

    fn map_position_to_window_position(&self, map_position: (i32, i32)) -> (f64, f64) {
        let (i, j) = map_position;
        let x = ((j as f64 - self.view_in_map_j) / self.view_in_map_width) * self.view_in_window_width + self.view_in_window_x;
        let y = ((i as f64 - self.view_in_map_i) / self.view_in_map_height) * self.view_in_window_height + self.view_in_window_y;
        (x, y)
    }

    fn is_in_rect<T: PartialOrd>(&self, position: (T, T), rect: (T, T, T, T)) -> bool {
        let (x, y) = position;
        let (x0, y0, x1, y1) = rect;
        if x > x1 {return false;}
        else if x < x0 {return false;}
        else if y > y1 {return false;}
        else if y < y0 {return false;}
        else {return true;}
    }

    fn is_in_window(&self, window_position: (f64, f64)) -> bool {
        let (x, y) = window_position;
        let window_rect = (0.0, 0.0, self.map_size as f64, self.map_size as f64);
        self.is_in_rect((x, y), window_rect)
    }

    fn is_in_map(&self, map_position: (i32, i32)) -> bool {
        let (i, j) = map_position;
        let map_rect = (0, 0, self.map_size as i32, self.map_size as i32);
        self.is_in_rect((i, j), map_rect)
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

    // Gameplay functions
    fn turn(&mut self) {
        for unit in self.unit_map.map.iter_mut() {
            if let Some(unit) = unit {
                unit.remaining_moves = unit.speed;
            }
        }

        // Change active player and reset active unit position
        self.active_player = (self.active_player + 1) % self.player_num;
        let active_player_base_position = self.players[self.active_player].base_position;
        self.view_in_map_width = 16.0;
        self.view_in_map_height = 16.0;
        self.look_at([active_player_base_position[0] as f64, active_player_base_position[1] as f64]);
        self.active_unit_position = None;
    }

    fn deactivate_active_unit(&mut self) {
        self.active_unit_position = None;
    }

    fn activate_unit(&mut self, new_active_unit_position: [usize; 2]) {
        // First deactivate active unit if there is one
        self.deactivate_active_unit();

        // Activate unit by storing its position
        self.active_unit_position = Some(new_active_unit_position);
    }

    // View-related functions
    fn look_at(&mut self, pos: [f64; 2]) {
        // Extract grid coordinates
        let (i, j) = (pos[0], pos[1]);

        // Clip coordinates
        let i = i.min(self.map_size as f64).max(0.0);
        let j = j.min(self.map_size as f64).max(0.0);

        // Shift view to look at specified grid position
        self.view_in_map_i = i - self.view_in_map_height / 2.0;
        self.view_in_map_j = j - self.view_in_map_width / 2.0;
    }

    // Event and Update methods
    pub fn process_event(&mut self, event: Event) {
        // Get the latest mouse position
        if let Some(args) = event.mouse_cursor_args() {
            self.latest_mouse_position = Some(args);
        }

        // Mouse button pressed
        if let Some(Button::Mouse(mouse_button)) = event.press_args() {
            
            // Left mouse button pressed
            if mouse_button == MouseButton::Left {
                let latest_mouse_pos = self.latest_mouse_position.unwrap();

                let (mx, my) = (latest_mouse_pos[0], latest_mouse_pos[1]);
                let (mi, mj) = self.window_position_to_map_position((mx, my));

                if self.is_in_map((mi, mj)) {
                    self.pressed_map_cell = Some([mi as usize, mj as usize]);
                }
                else {
                    self.pressed_map_cell = None;
                }
            }

            // Right mouse button pressed
            // NOHTING
        }

        // Mouse button released
        if let Some(Button::Mouse(mouse_button)) = event.release_args() {
            
            // Left mouse button released
            if mouse_button == MouseButton::Left {
                let latest_mouse_position = self.latest_mouse_position.unwrap();

                // TODO : Make a function with that
                let (mx, my) = (latest_mouse_position[0], latest_mouse_position[1]);
                let (mi, mj) = self.window_position_to_map_position((mx, my));

                if self.is_in_map((mi, mj)) {
                    self.released_map_cell = Some([mi as usize, mj as usize]);
                }
                else {
                    self.released_map_cell = None;
                }

                // Click event
                if self.pressed_map_cell == self.released_map_cell {
                    if let Some(released_map_cell) = self.released_map_cell {
                        println!("Click on i={}, j={} ...", released_map_cell[0], released_map_cell[1]);

                        // Check if there is an active unit
                        if let Some(active_unit_position) = self.active_unit_position {
                            if let Some(active_unit) = &self.unit_map[(active_unit_position[0], active_unit_position[1])] {
                                println!("... while having active unit : {:?} ...", active_unit);
                                if let Some(underlying_unit) = &self.unit_map[(released_map_cell[0], released_map_cell[1])] {
                                    println!("... and clicking on unit : {:?}", underlying_unit);
                                    // Same unit --> Deactivation of unit
                                    if underlying_unit == active_unit {
                                        self.deactivate_active_unit();
                                    }
                                    // Unit of other player --> Possible attacks
                                    else if active_unit.player != underlying_unit.player {
                                        println!("Possible attacks of current active unit {:?} against {:?}", active_unit, underlying_unit);
                                    }
                                    // Another unit of the same player --> Deactivation and Activation of other unit
                                    else {
                                        self.activate_unit(underlying_unit.position);
                                    }
                                }
    
                                // No underlying unit and active unit --> Possible moves
                                else {
                                    let d = (released_map_cell[0] as i32 - active_unit_position[0] as i32).abs() + (released_map_cell[1] as i32 - active_unit_position[1] as i32).abs();
                                    if d <= active_unit.remaining_moves as i32 {
                                        // Make the moves
                                        self.unit_map[(released_map_cell[0], released_map_cell[1])] = self.unit_map[(active_unit_position[0], active_unit_position[1])];
                                        self.unit_map[(active_unit_position[0], active_unit_position[1])] = None;

                                        // Update active unit remaining moves and position
                                        if let Some(active_unit) = &mut self.unit_map[(released_map_cell[0], released_map_cell[1])] {
                                            active_unit.remaining_moves -= d;
                                            active_unit.position = released_map_cell;
                                            self.territory_map[(released_map_cell[0], released_map_cell[1])] = active_unit.player;
                                        }

                                        // Update active unit positition
                                        self.active_unit_position = Some(released_map_cell);
                                    }
                                }
                            }
                        }

                        // No active unit 
                        else {
                            // Activation underlying unit
                            if let Some(underlying_unit) = &self.unit_map[(released_map_cell[0], released_map_cell[1])] {
                                println!("Click on unit : {:?}", underlying_unit);
                                if underlying_unit.player == self.active_player {
                                    self.active_unit_position = Some(underlying_unit.position);
                                }
                            }
                        }
                    }   
                }

                self.pressed_map_cell = None;
            }

            // Right mouse button released
            // NOTHING
        }

        // Keyboard button pressed
        if let Some(Button::Keyboard(key)) = event.press_args() {
            let view_move_in_view_size_ratio: f64 = 0.1;
            let view_move_i = self.view_in_map_height * view_move_in_view_size_ratio;
            let view_move_j = self.view_in_map_width * view_move_in_view_size_ratio;
            let n: f64 = 2.0;
            match key {
                Key::Up | Key::Z => {
                    self.view_in_map_i = (self.view_in_map_i - view_move_i).max(-self.view_in_map_height / n);
                },
                Key::Down | Key::S => {
                    self.view_in_map_i = (self.view_in_map_i + view_move_i).min(self.map_size as f64 - (1.0 - (1.0 / n)) * self.view_in_map_height);
                },
                Key::Left | Key::Q => {
                    self.view_in_map_j = (self.view_in_map_j - view_move_j).max(-self.view_in_map_width / n);
                },
                Key::Right | Key::D => {
                    self.view_in_map_j = (self.view_in_map_j + view_move_j).min(self.map_size as f64 - (1.0 - (1.0 / n)) * self.view_in_map_height);
                },
                Key::Space => {
                    self.turn();
                },
                Key::R => {
                    self.view_in_map_width = self.map_size as f64;
                    self.view_in_map_height = self.map_size as f64;
                    self.look_at([self.map_size as f64 / 2.0, self.map_size as f64 / 2.0]);
                }
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

        let (view_in_map_i1, view_in_map_j1, view_in_map_i2, view_in_map_j2) = self.visible_map_bounds();

        for i in view_in_map_i1..view_in_map_i2 {
            let y = self.i_to_y(i);
            for j in view_in_map_j1..view_in_map_j2 {
                let x = self.j_to_x(j);
                let transform = c.transform.trans(x, y);

                // Draw each grid cell
                rectangle(self.h_to_color(self.height_map[(i as usize, j as usize)], false), cell, transform, self.gl.as_mut().unwrap());
            }
        }

        // Draw pressed grid cells
        if let Some(pressed_map_cell) = self.pressed_map_cell {
            let x = self.j_to_x(pressed_map_cell[1] as i32);
            let y = self.i_to_y(pressed_map_cell[0] as i32);
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
            if let Some(active_unit) = &self.unit_map[(active_unit_position[0], active_unit_position[1])] {
                // Compute cell dimensions in pixel and define reachable mask shape
                let (cell_pix_width, cell_pix_height) = self.cell_pixel();
                let reachable_cell = rectangle_by_corners(0.0, 0.0, cell_pix_width, cell_pix_height);
                let (view_in_map_i1, view_in_map_j1, view_in_map_i2, view_in_map_j2) = self.visible_map_bounds();

                // Draw each reachable cell by active unit
                let (pi, pj) = (active_unit_position[0] as i32, active_unit_position[1] as i32);
                let (ibeg, iend) = (std::cmp::max(pi - active_unit.remaining_moves, view_in_map_i1), std::cmp::min(pi + active_unit.remaining_moves, view_in_map_i2-1));
                let (jbeg, jend) = (std::cmp::max(pj - active_unit.remaining_moves, view_in_map_j1), std::cmp::min(pj + active_unit.remaining_moves, view_in_map_j2-1));
                for i in ibeg..=iend {
                    for j in jbeg..=jend {
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

    fn render_buildings(&mut self, c: Context) {
        // Compute cell dimensions in pixel
        let (cell_pix_width, cell_pix_height) = self.cell_pixel();
        let (view_in_map_i1, view_in_map_j1, view_in_map_i2, view_in_map_j2) = self.visible_map_bounds();

        let cell_padding_ratio = 1.0 / 4.0;
        let building_pix_width = cell_pix_width * (1.0 - cell_padding_ratio * 2.0);
        let building_pix_height = cell_pix_height * (1.0 - cell_padding_ratio * 2.0);
        let rectangle = [
            cell_pix_width * cell_padding_ratio, cell_pix_height * cell_padding_ratio, 
            building_pix_width, building_pix_height
        ];
        let border_padding_ratio = 1.0 / 10.0;
        let building_rectangle = Rectangle {
            color: [1.0, 1.0, 1.0, 1.0],
            shape: graphics::rectangle::Shape::Square,
            border: Some(graphics::rectangle::Border {
                color: [0.0, 0.0, 0.0, 1.0],
                radius: (building_pix_width / 2.0) * border_padding_ratio
            })
        };

        // Draw units
        for i in view_in_map_i1..view_in_map_i2 {
            for j in view_in_map_j1..view_in_map_j2 {
                if let Some(building) = &self.building_map[(i as usize, j as usize)] {
                    building_rectangle
                        .color(self.players[building.player].principal_color)
                        .border(graphics::rectangle::Border {
                            color: self.players[building.player].secondary_color,
                            radius: (building_pix_width / 2.0) * border_padding_ratio})
                        .draw(
                            rectangle, 
                            &draw_state::DrawState::default(), 
                            c.transform.trans(self.j_to_x(j), self.i_to_y(i)), 
                            self.gl.as_mut().unwrap()
                        );
                }
            }
        }
    }

    fn render_units(&mut self, c: Context) {
        // Compute cell dimensions in pixel and compute visible map bounds
        let (cell_pix_width, cell_pix_height) = self.cell_pixel();
        let visible_map_bounds = self.visible_map_bounds();
        let (view_in_map_i1, view_in_map_j1, view_in_map_i2, view_in_map_j2) = visible_map_bounds;

        let cell_padding_ratio = 1.0 / 4.0;
        let unit_pix_width = cell_pix_width * (1.0 - cell_padding_ratio * 2.0);
        let unit_pix_height = cell_pix_height * (1.0 - cell_padding_ratio * 2.0);
        let rectangle = [
            cell_pix_width * cell_padding_ratio, cell_pix_height * cell_padding_ratio, 
            unit_pix_width, unit_pix_height
        ];
        let border_padding_ratio = 1.0 / 10.0;
        let unit_ellipse = Ellipse {
            color: [1.0, 1.0, 1.0, 1.0],
            border: Some(graphics::ellipse::Border {
                color: [1.0, 1.0, 1.0, 1.0],
                radius: (unit_pix_width / 2.0) * border_padding_ratio
            }),
            resolution: 32
        };

        // Draw units
        for i in view_in_map_i1..view_in_map_i2 {
            for j in view_in_map_j1..view_in_map_j2 {
                if let Some(unit) = &self.unit_map[(i as usize, j as usize)] {
                    let (x, y) = self.map_position_to_window_position((i, j));
                    unit_ellipse
                        .color(self.players[unit.player].principal_color)
                        .border(graphics::ellipse::Border {
                            color: self.players[unit.player].secondary_color,
                            radius: (unit_pix_width / 2.0) * border_padding_ratio})
                        .draw(
                            rectangle, &draw_state::DrawState::default(), 
                            c.transform.trans(x, y), self.gl.as_mut().unwrap()
                        );
                }
            }
        }

        // Draw marker on active unit if there is one and if it is visible
        if let Some(active_unit_position) = self.active_unit_position {
            if self.is_in_rect((active_unit_position[0] as i32, active_unit_position[1] as i32), visible_map_bounds) {
                let cell_padding_ratio = 1.0 / 2.5;
                let marker_pix_width = cell_pix_width * (1.0 - cell_padding_ratio * 2.0);
                let marker_pix_height = cell_pix_height * (1.0 - cell_padding_ratio * 2.0);
                let rectangle = [
                    cell_pix_width * cell_padding_ratio, cell_pix_height * cell_padding_ratio, 
                    marker_pix_width, marker_pix_height
                ];
                let active_unit_marker = Ellipse {
                    color: [0.0, 0.0, 0.0, 1.0],
                    border: None,
                    resolution: 32
                };
                let (x, y) = self.map_position_to_window_position((active_unit_position[0] as i32, active_unit_position[1] as i32));

                active_unit_marker.draw(
                    rectangle, &draw_state::DrawState::default(), 
                    c.transform.trans(x, y), self.gl.as_mut().unwrap()
                );
            }
        }
    }

    fn render_territory(&mut self, c: Context) {
        // Compute cell dimensions in pixel
        let (cell_pix_width, cell_pix_height) = self.cell_pixel();
        let (view_in_map_i1, view_in_map_j1, view_in_map_i2, view_in_map_j2) = self.visible_map_bounds();
        let font_size = (cell_pix_height / 4.0).floor() as u32;

        for i in view_in_map_i1..view_in_map_i2 {
            for j in view_in_map_j1..view_in_map_j2 {
                let player = self.territory_map[(i, j)];
                if player < self.player_num {
                    // Get principal and secondary colors for drawing player text
                    let principal_color = self.players[player].principal_color;
                    let _secondary_color = self.players[player].secondary_color;
                    let player_text = Text {
                        color: principal_color,
                        font_size: font_size,
                        round: false
                    };

                    let (x, y) = self.map_position_to_window_position((i, j));

                    player_text.draw(player.to_string().as_str() , self.glyphs.as_mut().unwrap(),
                                     &draw_state::DrawState::default(), c.transform.trans(x, y + font_size as f64),
                                     self.gl.as_mut().unwrap());
                }
            }
        }

        // Initialize a processed/visited map
        // let mut processed_map = Map::<bool>::new(self.map_size, self.map_size, false);

        // // Contour generation
        // let mut contours_positions: Vec<Vec<(i32, i32)>> = Vec::new();
        // for i in 0..self.map_size {
        //     for j in 0..self.map_size {
        //         let player = self.territory_map[(i, j)];
        //         if player < self.player_num && !processed_map[(i, j)] {
        //             // Launch contour detection from this point
        //             let init_position = (i as i32, j as i32);
        //             let mut current_position: (i32, i32) = (i as i32, j as i32);
        //             processed_map[current_position] = true;
        //             let mut contour_positions = Vec::<(i32, i32)>::new();
        //             contour_positions.push(init_position);
        //             let mut current_direction: (i32, i32) = (1, 0);

        //             // Mark init position as processed
        //             processed_map[current_position] = true;

        //             // Generate contours
        //             loop {
        //                 let mut has_move_right: bool = false;
        //                 let mut has_move_ahead: bool = false;
        //                 let mut has_move_left: bool = false;
        //                 let mut has_move_behind: bool = false;

        //                 let right_direction = (current_direction.1, -current_direction.0);
        //                 let left_direction = (-current_direction.1, current_direction.0);
        //                 let behind_direction = (-current_direction.0, -current_direction.1);

        //                 // Mark current position as processed/visited
        //                 processed_map[current_position] = true;

        //                 // Look to the right
        //                 let right_position = ((current_position.0 + right_direction.0), (current_position.1 + right_direction.1));
        //                 if right_position.0 >= 0 && right_position.0 < self.map_size as i32 && right_position.1 >= 0 && right_position.1 < self.map_size as i32 {
        //                     if self.territory_map[right_position] == player {
        //                         // Update current position and direction
        //                         current_position = right_position;
        //                         current_direction = right_direction;
        //                         has_move_right = true;
        //                     }
        //                 }

        //                 if !has_move_right {
        //                     // Generate new contour position
        //                     let last_contour_position = contour_positions[contour_positions.len() - 1];
        //                     let new_contour_position = (last_contour_position.0 + current_direction.0, last_contour_position.1 + current_direction.1);
        //                     contour_positions.push(new_contour_position);
        //                     if contour_positions[0] == new_contour_position {
        //                         break;
        //                     }

        //                     // Right impossible --> Look ahead
        //                     let ahead_position = ((current_position.0 + current_direction.0), (current_position.1 + current_direction.1));
        //                     if ahead_position.0 >= 0 && ahead_position.0 < self.map_size as i32 && ahead_position.1 >= 0 && ahead_position.1 < self.map_size as i32 {
        //                         if self.territory_map[ahead_position] == player {
        //                             // Update current position (direction doesn't change)
        //                             current_position = ahead_position;
        //                             has_move_ahead = true;
        //                         }
        //                     }
        //                 }

        //                 if !has_move_ahead && !has_move_right {
        //                     // Generate new contour position
        //                     let last_contour_position = contour_positions[contour_positions.len() - 1];
        //                     let new_contour_position = (last_contour_position.0 + left_direction.0, last_contour_position.1 + left_direction.1);
        //                     contour_positions.push(new_contour_position);
        //                     if contour_positions[0] == new_contour_position {
        //                         break;
        //                     }

        //                     // Ahead impossible --> Look to the left
        //                     let left_position = ((current_position.0 + left_direction.0), (current_position.1 + left_direction.1));
        //                     if left_position.0 >= 0 && left_position.0 < self.map_size as i32 && left_position.1 >= 0 && left_position.1 < self.map_size as i32 {
        //                         if self.territory_map[left_position] == player {
        //                             // Update current position and direction
        //                             current_position = left_position;
        //                             current_direction = left_direction;
        //                             has_move_left = true;
        //                         }
        //                     }
        //                 }

        //                 if !has_move_left && !has_move_ahead && !has_move_right {
        //                     // Generate new contour position
        //                     let last_contour_position = contour_positions[contour_positions.len() - 1];
        //                     let new_contour_position = (last_contour_position.0 + behind_direction.0, last_contour_position.1 + behind_direction.1);
        //                     contour_positions.push(new_contour_position);
        //                     if contour_positions[0] == new_contour_position {
        //                         break;
        //                     }

        //                     // Look behind
        //                     let behind_position = ((current_position.0 + behind_direction.0), (current_position.1 + behind_direction.1));
        //                     if behind_position.0 >= 0 && behind_position.0 < self.map_size as i32 && behind_position.1 >= 0 && behind_position.1 < self.map_size as i32 {
        //                         if self.territory_map[behind_position] == player {
        //                             // Update current position and direction
        //                             current_position = behind_position;
        //                             current_direction = behind_direction;
        //                             has_move_behind = true;
        //                         }
        //                     }
        //                 }

        //                 if !has_move_behind && !has_move_left && !has_move_ahead && !has_move_right {
        //                     // Generate new contour position
        //                     let last_contour_position = contour_positions[contour_positions.len() - 1];
        //                     let new_contour_position = (last_contour_position.0 + right_direction.0, last_contour_position.1 + right_direction.1);
        //                     contour_positions.push(new_contour_position);
        //                     if contour_positions[0] == new_contour_position {
        //                         break;
        //                     }
        //                 }
        //             }

        //             contours_positions.push(contour_positions);
        //         }
        //     } 
        // }

        // // Draw contours
        // for contour_positions in contours_positions.iter() {
        //     for i in 0..contour_positions.len() - 1 {
        //         let (ibeg, jbeg) = contour_positions[i];
        //         let (iend, jend) = contour_positions[i+1];

        //         let line = [0.0, 0.0, self.j_to_x(jend) - self.j_to_x(jbeg), self.i_to_y(iend) - self.i_to_y(ibeg)];
        //         let filled_line = Line {
        //             color: [0.0, 0.0, 0.0, 1.0],
        //             radius: cell_pix_height / 32.0,
        //             shape: graphics::line::Shape::Square
        //         };

        //         filled_line.draw(line, &draw_state::DrawState::default(), c.transform.trans(self.j_to_x(jbeg), self.i_to_y(ibeg)), self.gl.as_mut().unwrap())
        //     }
        // }

        // let territorry_cell_width = cell_pix_width;
        // let territorry_cell_height = cell_pix_height;
        // let border_padding_ratio = 1.0 / 32.0;
        // let outline_radius = territorry_cell_height * border_padding_ratio;
        // let rectangle = [outline_radius, outline_radius, territorry_cell_width - 2.0*outline_radius, territorry_cell_height - 2.0*outline_radius];
        // let territory_mask_rectangle = Rectangle {
        //     color: [0.0, 0.0, 0.0, 0.0],
        //     // shape: graphics::rectangle::Shape::Square,
        //     shape: graphics::rectangle::Shape::Round(outline_radius, 32),
        //     border: Some(graphics::rectangle::Border {
        //         color: [0.0, 0.0, 0.0, 1.0],
        //         radius: 0.0
        //     })
        // };

        // // Draw units
        // for i in view_in_map_i1..view_in_map_i2 {
        //     for j in view_in_map_j1..view_in_map_j2 {
        //         if let Some(player) = &self.territory_map[(i as usize, j as usize)] {
        //             let mut territory_mask_color = self.players[*player].principal_color;
        //             territory_mask_color[3] = 0.8;
        //             territory_mask_rectangle
        //                 .border(graphics::rectangle::Border {
        //                     color: territory_mask_color,
        //                     radius: outline_radius})
        //                 .draw(
        //                     rectangle, 
        //                     &draw_state::DrawState::default(), 
        //                     c.transform.trans(self.j_to_x(j), self.i_to_y(i)), 
        //                     self.gl.as_mut().unwrap()
        //                 );
        //         }
        //     }
        // }
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

        // Render units and buildings
        self.render_buildings(c);
        self.render_units(c);

        // Render territory
        self.render_territory(c);

        // End the drawing pipeline
        self.gl.as_mut().unwrap().draw_end();
    }
}
