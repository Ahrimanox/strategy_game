use std::collections::VecDeque;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

use piston::input::{
    RenderArgs, 
    RenderEvent, 
    Event, 
    MouseCursorEvent, 
    PressEvent, 
    Button, 
    MouseButton, 
    ReleaseEvent, 
    MouseScrollEvent
};

use piston::input::keyboard::Key;

use opengl_graphics::{
    GlGraphics, 
    GlyphCache
};

use graphics::*;
use graphics::rectangle::rectangle_by_corners;

use graphics::text::Text;
use graphics::rectangle::Rectangle;
use graphics::ellipse::Ellipse;

use crate::utils::{
    is_in_rect
};

use crate::map::{
    Map, 
    diamond_square, 
    noise_map
};

pub use crate::player::{
    Unit, 
    Building, 
    Player
};

use crate::distance::{
    NullDistance2D, 
    EuclideanDistanceWHeight2D
};
use crate::constraint::{
    TerrainConstraint, 
    UnitConstraint, 
    BuildingConstraint
};
use crate::path_planning::astar_2d_map;

// Structure used to holding terrain information
#[derive(Debug, Clone)]
pub struct Terrain {
    pub name: String,
    pub color: [f32; 4],
    pub height_interval: (f64, f64)
}

impl Default for Terrain {
    fn default() -> Self {Terrain {name: String::from("None"), color: [0.0, 0.0, 0.0, 0.0], height_interval: (-1.0, -1.0)}}
}

impl PartialEq for Terrain {
    fn eq(&self, other: &Self) -> bool {
        if self.name != other.name {false}
        else if self.color != other.color {false}
        else if self.height_interval != other.height_interval {false}
        else {true}
    }
}

#[derive(Default)]
pub struct Game<'g> {

    // Rendering attributes
    pub gl: Option<GlGraphics>,
    pub glyphs: Option<GlyphCache<'g>>,

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
    pub unit_default_speed: f64,
    pub player_num: usize,

    // Player vector
    pub players: Vec<Player>,

    // Building map (hold building for each position in the map
    pub building_map: Rc<RefCell<Map<Weak<RefCell<Building>>>>>,

    // Unit map (hold Unit for each position in the map
    pub unit_map: Rc<RefCell<Map<Weak<RefCell<Unit>>>>>,

    // Territory map
    pub territory_map: Map<usize>,

    // Height map
    pub height_map: Map<f64>,

    // Terrain map
    pub terrain_map: Rc<RefCell<Map<Weak<Terrain>>>>,

    pub terrains: Vec<Rc<Terrain>>,
    pub color_ramp_value: Vec<f64>,
    pub color_ramp_color: Vec<[f32; 4]>,

    pub active_player: usize,
    pub active_unit: Weak<RefCell<Unit>>,
    pub active_unit_planned_path: Option<VecDeque<(i32, i32, f64)>>,
    pub current_mouse_position: Option<[f64; 2]>,

    pub current_underlying_cell: Option<[usize; 2]>,
    pub pressed_map_cell: Option<[usize; 2]>,
    pub released_map_cell: Option<[usize; 2]>,
}

impl<'g> Game<'g> {
    
    // Init game
    pub fn init(&mut self) {

        // Compute map size
        self.map_size = (2 as usize).pow(4+self.map_size_level)+1;

        // Initialize unit map and building map with None
        self.unit_map = Rc::new(RefCell::new(Map::<Weak<RefCell<Unit>>>::new(self.map_size, self.map_size, Weak::new())));
        self.building_map = Rc::new(RefCell::new(Map::<Weak<RefCell<Building>>>::new(self.map_size, self.map_size, Weak::new())));
        self.territory_map = Map::new(self.map_size, self.map_size, self.player_num);

        // Generation of playable map
        self.generate_map();

        // Initialize players 
        // TODO : Initialize base position for all players with clever algorithm
        self.players = Vec::new();
        self.players.push(
            Player::new(
                0, 
                (0, 0), 
                [1.0, 0.0, 0.0, 1.0], 
                [0.0, 0.0, 0.0, 1.0]
            )
        );
        self.players.push(
            Player::new(
                1, 
                (self.map_size - 1, self.map_size - 1), 
                [0.0, 0.0, 1.0, 1.0], 
                [0.0, 0.0, 0.0, 1.0]
            )
        );

        // Add reference to player buildings in buildings map
        for player in &self.players {
            for building in &player.buildings {
                (self.building_map.borrow_mut())[building.borrow().position] = Rc::downgrade(&building);
                self.territory_map[building.borrow().position] = building.borrow().player;
            }
        }

        // Add first units
        // TODO : Make a function to generate unit near to base building
        self.players[0].units.push(
            Rc::new(
                RefCell::new(
                    Unit {
                        player: 0,
                        position: (0, 1),
                        damage: 1.0,
                        health: 1.0,
                        speed: self.unit_default_speed,
                        remaining_moves: self.unit_default_speed
                    }
                )
            )
        );
        self.players[0].units.push(
            Rc::new(
                RefCell::new(
                    Unit {
                        player: 0,
                        position: (1, 0),
                        damage: 1.0,
                        health: 1.0,
                        speed: self.unit_default_speed,
                        remaining_moves: self.unit_default_speed
                    }
                )
            )
        );

        self.players[1].units.push(
            Rc::new(
                RefCell::new(
                    Unit {
                        player: 1,
                        position: (self.map_size - 1, self.map_size - 2),
                        damage: 1.0,
                        health: 1.0,
                        speed: self.unit_default_speed,
                        remaining_moves: self.unit_default_speed
                    }
                )
            )
        );
        self.players[1].units.push(
            Rc::new(
                RefCell::new(
                    Unit {
                        player: 1,
                        position: (self.map_size - 2, self.map_size - 1),
                        damage: 1.0,
                        health: 1.0,
                        speed: self.unit_default_speed,
                        remaining_moves: self.unit_default_speed
                    }
                )
            )
        );
       
        // Add reference to player units in units map
        for player in &self.players {
            for unit in &player.units {
                (self.unit_map.borrow_mut())[unit.borrow().position] = Rc::downgrade(unit);
                self.territory_map[unit.borrow().position] = unit.borrow().player;
            }
        }

        // Let the first player be the current active player
        self.active_player = 0;

        // Look at active player base position
        self.look_at_active_user_base();
    }

    // Map generation function
    pub fn generate_map(&mut self) {

        // Generate procedurally height map
        self.height_map = Map::new(self.map_size, self.map_size, 0.0);
        let lacunarity = 2.0;
        self.height_map = noise_map(
            self.map_size,
            16,
            lacunarity,
            lacunarity, 
            1.0, 
            false
        );

        // Assign Terrain to map cell according to cell height
        // TODO : Pass terrain information by configuration file to reduce code complexity
        let terrain_vec: Vec<Terrain> = vec![
            Terrain {name: String::from("DeepWater"), color: [0.007, 0.176, 0.357, 1.0], height_interval: (0.0, 0.2)},
            Terrain {name: String::from("CoastalWater"), color: [0.051, 0.286, 0.404, 1.0], height_interval: (0.2, 0.275)},
            Terrain {name: String::from("Sand"), color: [0.98, 0.84, 0.45, 1.0], height_interval: (0.275, 0.3)},
            Terrain {name: String::from("Grass"), color: [0.204, 0.412, 0.180, 1.0], height_interval: (0.3, 0.7)},
            Terrain {name: String::from("Mountain"), color: [0.557, 0.541, 0.341, 1.0], height_interval: (0.7, 0.90)},
            Terrain {name: String::from("SnowyPeak"), color: [1.0, 1.0, 1.0, 1.0], height_interval: (0.90, 1.0 + 0.01)},
        ];

        self.terrains.extend(terrain_vec.into_iter().map(|x| Rc::new(x)));
        
        self.terrain_map = Rc::new(RefCell::new(Map::<Weak<Terrain>>::new(self.map_size, self.map_size, Weak::new())));
        for i in 0..self.map_size {
            for j in 0..self.map_size {
                for terrain in self.terrains.iter() {
                    let height = self.height_map[(i, j)];
                    if height >= terrain.height_interval.0 && height < terrain.height_interval.1 {
                        self.terrain_map.borrow_mut()[(i, j)] = Rc::downgrade(terrain);
                    }
                }
            }
        }
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

    fn is_in_window(&self, window_position: (f64, f64)) -> bool {
        let (x, y) = window_position;
        let window_rect = (0.0, 0.0, self.map_size as f64, self.map_size as f64);
        is_in_rect((x, y), window_rect, true)
    }

    fn is_in_map(&self, map_position: (i32, i32)) -> bool {
        let (i, j) = map_position;
        let map_rect = (0, 0, self.map_size as i32, self.map_size as i32);
        is_in_rect((i, j), map_rect, false)
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
                    let alpha: f32 = 1.0 - ((h - lvalue) / (rvalue - lvalue)) as f32;
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
    fn is_there_unit_on(&self, pos: (usize, usize)) -> bool {
        self.unit_map.borrow()[pos].upgrade().is_some()
    }

    fn is_there_building_on(&self, pos: (usize, usize)) -> bool {
        self.building_map.borrow()[pos].upgrade().is_some()
    }

    fn unit(&self, pos: (usize, usize)) -> Weak<RefCell<Unit>> {
        self.unit_map.borrow()[pos].clone()
    }

    fn building(&self, pos: (usize, usize)) -> Weak<RefCell<Building>> {
        self.building_map.borrow()[pos].clone()
    }

    fn turn(&mut self) {

        // Update active player and reset active unit
        self.active_player = (self.active_player + 1) % self.player_num;

        self.deactivate_active_unit();

        // self.look_at_active_user_base();
        self.look_at_overview();

        // Restore all moves of current active player units
        for player in &self.players {
            for unit in &player.units {
                let speed = unit.borrow().speed;
                unit.borrow_mut().remaining_moves = speed;
            }
        }
    }

    fn deactivate_active_unit(&mut self) {
        self.active_unit_planned_path = None;
        self.active_unit = Weak::new();
    }

    fn activate_unit(&mut self, new_active_unit_position: (usize, usize)) {
        // First deactivate active unit if there is one
        self.deactivate_active_unit();

        // Activate unit by storing its position
        self.active_unit = self.unit_map.borrow()[new_active_unit_position].clone();
    }
        

    // FIXME : Fix bugs when unit moves, losing track of active unit and its position
    // TODO : Reformat this part of code if possible
    fn moves(&mut self, destination: (usize, usize)) {

        // Get active unit position
        if let Some(active_unit) = self.active_unit.upgrade() {
            
            // Make the moves
            self.unit_map.borrow_mut()[destination] = self.active_unit.clone();
            self.unit_map.borrow_mut()[active_unit.borrow().position] = Weak::new();
            
            // Update active unit position attribute
            active_unit.borrow_mut().position = destination;

            self.takes_territory(destination);
        }
    }

    fn takes_territory(&mut self, territory_position: (usize, usize)) {
        self.territory_map[territory_position] = self.active_player;
    }

    fn execute_planned_path(&mut self) {
        
        // Check if there is an active unit
        if let Some(active_unit) = self.active_unit.upgrade() {

            // If there is an active planned path -> Execute it
            let mut previous_cost = 0.0;
            if let Some(active_unit_planned_path) = &self.active_unit_planned_path {
                for (i, j, cost) in active_unit_planned_path.clone().iter().skip(1) {

                    let current_destination = (*i as usize, *j as usize);
                
                    // Check if move is possible by checking updated remaining move
                    if active_unit.borrow().remaining_moves >= (*cost - previous_cost) {
                        
                        if self.is_there_unit_on(current_destination) {
                            
                            let unit_on_destination = self.unit(current_destination).upgrade().unwrap();
                            
                            // Actions that a unit can do to other allied units
                            // TODO : implement rule for this case
                            if unit_on_destination.borrow().player == active_unit.borrow().player {
                                break;
                            }
                            // Attack
                            else {
                                unit_on_destination.borrow_mut().health -= active_unit.borrow().damage;
                                active_unit.borrow_mut().remaining_moves -= *cost - previous_cost;
                                self.players[unit_on_destination.borrow().player].purge_dead_units();
                                break;
                            }
                        }
                        else if self.is_there_building_on(current_destination) {
                            
                            let building_on_destination = self.building(current_destination).upgrade().unwrap();
                            
                            // Actions that a unit can do to other allied units
                            // TODO : implement rule for this case
                            if building_on_destination.borrow().player == active_unit.borrow().player {
                                break;
                            }
                            // Attack
                            else {
                                building_on_destination.borrow_mut().health -= active_unit.borrow().damage;
                                active_unit.borrow_mut().remaining_moves -= *cost - previous_cost;
                                self.players[building_on_destination.borrow().player].purge_dead_buildings();
                                break;
                            }
                        }
                        else {
                            self.moves(current_destination);

                            active_unit.borrow_mut().remaining_moves -= *cost - previous_cost;
                        }
                        
                        previous_cost = *cost;
                    }
                }
            }
            
            // Reset active unit planned path
            self.active_unit_planned_path = None;
        }
    }

    // View-related functions
    fn view_center(&self) -> [f64; 2] {
        [
            self.view_in_map_i + self.view_in_map_height / 2.0,
            self.view_in_map_j + self.view_in_map_width / 2.0
        ]
    }

    fn shift_view(&mut self, shift: (f64, f64)) {
        let frac =  1.0 / 2.0;
        self.view_in_map_i = 
            (self.view_in_map_i + shift.0)
            .max(-self.view_in_map_height * frac)
            .min(self.map_size as f64 - self.view_in_map_height * (1.0 - frac) - f64::EPSILON);
        
        self.view_in_map_j = 
            (self.view_in_map_j + shift.1)
            .max(-self.view_in_map_width * frac)
            .min(self.map_size as f64 - self.view_in_map_width * (1.0 - frac) - f64::EPSILON);
    }

    fn look_at(&mut self, pos: (f64, f64)) {
        
        // Shift view
        self.shift_view(
            (
                (pos.0 - self.view_in_map_height / 2.0) - self.view_in_map_i,
                (pos.1 - self.view_in_map_width / 2.0) - self.view_in_map_j
            )
        );
    }

    fn zoom(&mut self, scroll_factor: f64) {
        let view_center = self.view_center();
        let (view_center_in_map_i, view_center_in_map_j) = (view_center[0], view_center[1]);

        if scroll_factor == 1.0 {
            self.view_in_map_height = (self.view_in_map_height / 2.0).max(3.0);
            self.view_in_map_width = (self.view_in_map_width / 2.0).max(3.0);
        }
        else if scroll_factor == -1.0 {
            self.view_in_map_height = (self.view_in_map_height * 2.0).min(2.0 * self.map_size as f64);
            self.view_in_map_width = (self.view_in_map_width * 2.0).min(2.0 * self.map_size as f64);
        }

        self.view_in_map_i = view_center_in_map_i - self.view_in_map_height / 2.0;
        self.view_in_map_j = view_center_in_map_j - self.view_in_map_width / 2.0;
    }

    fn look_at_overview(&mut self) {
        self.view_in_map_width = self.map_size as f64;
        self.view_in_map_height = self.map_size as f64;
        self.look_at((self.map_size as f64 / 2.0, self.map_size as f64 / 2.0));
    }

    fn look_at_cell(&mut self, cell: (usize, usize)) {
        self.look_at((cell.0 as f64 + 0.5, cell.1 as f64 + 0.5));
    }

    fn look_at_active_user_base(&mut self) {
        let base_pos = self.players[self.active_player].buildings[0].borrow().position;
        self.look_at_cell(base_pos);
    }

    // Event and Update methods
    // TODO : Collect differents input and store them in an appropriate structure
    pub fn process_event(&mut self, event: Event) {

        // Update the current mouse position and underlying cell (if possible)
        if let Some(args) = event.mouse_cursor_args() {
            self.current_mouse_position = Some(args);

            // Get underlying map cell position
            let (mx, my) = (args[0], args[1]);
            let (mi, mj) = self.window_position_to_map_position((mx, my));

            if self.is_in_map((mi, mj)) {
                self.current_underlying_cell = Some([mi as usize, mj as usize]);
            }
            else {
                self.current_underlying_cell = None;
            }
        }
        else {
            self.current_mouse_position = None;
        }

        // Mouse button pressed
        if let Some(Button::Mouse(mouse_button)) = event.press_args() {
            
            // Left mouse button pressed
            if mouse_button == MouseButton::Left {
                
                // If there is underlying map cell -> Press on
                if let Some(current_underlying_cell) = self.current_underlying_cell {
                    self.pressed_map_cell = Some(current_underlying_cell)
                }
                else {
                    self.pressed_map_cell = None;
                }
            }
        }

        // Mouse button released
        if let Some(Button::Mouse(mouse_button)) = event.release_args() {
            
            // Left mouse button released
            if mouse_button == MouseButton::Left {

                // If there is underlying map cell -> Release from
                if let Some(current_underlying_cell) = self.current_underlying_cell {
                    self.released_map_cell = Some(current_underlying_cell)
                }
                else {
                    self.released_map_cell = None;
                }

                // If pressed map cell and released map cell is equal -> CLICK EVENT
                if self.pressed_map_cell == self.released_map_cell {
                    if let Some(released_map_cell) = self.released_map_cell {
                        let cpos = (released_map_cell[0], released_map_cell[1]);
                        let (ci, cj) = (cpos.0 as i32, cpos.1 as i32);
                        
                        // Check if there is an active unit
                        if self.active_unit.upgrade().is_some() {

                            // Compute "optimal" path from active unit position to the pointed position
                            let active_unit_position = self.active_unit.upgrade().unwrap().borrow().position;
                            let start = (active_unit_position.0 as i32, active_unit_position.1 as i32);
                            let goal = (ci, cj);
                            let distance = EuclideanDistanceWHeight2D {
                                height_map: &self.height_map
                            };
                            let heuristic = NullDistance2D {};
                            let water_constraint = Box::new(TerrainConstraint {
                                terrain_map: Rc::downgrade(&self.terrain_map),
                                impractical_terrains: vec![
                                    Rc::downgrade(&self.terrains[0]),
                                    Rc::downgrade(&self.terrains[1])
                                ]
                            });

                            let unit_constraint = Box::new(
                                UnitConstraint {
                                    unit_map: Rc::downgrade(&self.unit_map)
                                }
                            );
                            let building_constraint = Box::new(
                                BuildingConstraint {
                                    building_map: Rc::downgrade(&self.building_map)
                                }
                            );

                            let path_res = astar_2d_map(
                                start, 
                                goal, 
                                (self.map_size as i32, self.map_size as i32), 
                                distance, 
                                heuristic,
                                vec![water_constraint, unit_constraint, building_constraint],
                                vec![],
                            );
                            if let Some(path) = path_res {
                                self.active_unit_planned_path = Some(path);
                            }
                            else {
                                self.active_unit_planned_path = None;
                                println!("Something wrong with path planning step");
                            }


                            // let d = (released_map_cell[0] as i32 - active_unit_position[0] as i32).abs() + (released_map_cell[1] as i32 - active_unit_position[1] as i32).abs();
                            // if d <= active_unit.remaining_moves as i32 {
                            //     // Make the moves
                            //     self.unit_map[(released_map_cell[0], released_map_cell[1])] = self.unit_map[(active_unit_position[0], active_unit_position[1])];
                            //     self.unit_map[(active_unit_position[0], active_unit_position[1])] = None;

                            //     // Update active unit remaining moves and position
                            //     if let Some(active_unit) = &mut self.unit_map[(released_map_cell[0], released_map_cell[1])] {
                            //         active_unit.remaining_moves -= d;
                            //         active_unit.position = released_map_cell;
                            //         self.territory_map[(released_map_cell[0], released_map_cell[1])] = active_unit.player;
                            //     }

                            //     // Update active unit positition
                            //     self.active_unit_position = Some(released_map_cell);
                            // }
                        }

                        // Activation underlying unit
                        if self.is_there_unit_on(cpos) {
                            
                            if self.unit(cpos).upgrade().unwrap().borrow().player == self.active_player {
                                self.activate_unit(cpos);
                                self.active_unit = self.unit_map.borrow()[cpos].clone();
                            }
                        }
                    }
                }

                self.pressed_map_cell = None;
                self.released_map_cell = None;
            }
        }

        // Keyboard button pressed
        if let Some(Button::Keyboard(key)) = event.press_args() {
            let view_move_in_view_size_ratio: f64 = 0.1;
            let view_move_i = self.view_in_map_height * view_move_in_view_size_ratio;
            let view_move_j = self.view_in_map_width * view_move_in_view_size_ratio;
            match key {
                Key::Up | Key::Z => {
                    self.shift_view((-view_move_i, 0.0));
                },
                Key::Down | Key::S => {
                    self.shift_view((view_move_i, 0.0));
                },
                Key::Left | Key::Q => {
                    self.shift_view((0.0, -view_move_j));
                },
                Key::Right | Key::D => {
                    self.shift_view((0.0, view_move_j));
                },
                Key::T => {
                    self.turn();
                },
                Key::Space => {
                    self.execute_planned_path();
                },
                Key::R => {
                    self.look_at_overview();
                },
                Key::B => {
                    self.look_at_active_user_base();
                },
                Key::G => {
                    self.generate_map();
                },
                Key::H => {
                    self.height_map = diamond_square(self.map_size);
                }
                _ => {}
            }
        }

        // Mouse scroll event
        if let Some(args) = event.mouse_scroll_args() {
            let (_scroll_x, scroll_y) = (args[0], args[1]);
            self.zoom(scroll_y);
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
                let terrain = &self.terrain_map.borrow()[(i, j)];
                if let Some(terrain) = terrain.upgrade() {
                    rectangle(
                        terrain.color, 
                        cell, 
                        transform, 
                        self.gl.as_mut().unwrap()
                    );
                }
                
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

    // TODO : Adapt this old function to draw shortest-path tree provided by future Dijkstra implementation
    // fn render_unit_reachable_cells(&mut self, c: Context) {
    //     // Draw reachable mask at reachable cells by active unit if there is an active one
    //     if let Some(active_unit_position) = self.active_unit_position {
    //         if let Some(active_unit) = &self.unit_map[(active_unit_position[0], active_unit_position[1])] {
    //             // Compute cell dimensions in pixel and define reachable mask shape
    //             let (cell_pix_width, cell_pix_height) = self.cell_pixel();
    //             let reachable_cell = rectangle_by_corners(0.0, 0.0, cell_pix_width, cell_pix_height);
    //             let (view_in_map_i1, view_in_map_j1, view_in_map_i2, view_in_map_j2) = self.visible_map_bounds();

    //             // Draw each reachable cell by active unit
    //             let (pi, pj) = (active_unit_position[0] as i32, active_unit_position[1] as i32);
    //             let (ibeg, iend) = (std::cmp::max(pi - active_unit.remaining_moves, view_in_map_i1), std::cmp::min(pi + active_unit.remaining_moves, view_in_map_i2-1));
    //             let (jbeg, jend) = (std::cmp::max(pj - active_unit.remaining_moves, view_in_map_j1), std::cmp::min(pj + active_unit.remaining_moves, view_in_map_j2-1));
    //             for i in ibeg..=iend {
    //                 for j in jbeg..=jend {
    //                     let d = (pi - i).abs() + (pj - j).abs();
    //                     if d as f64 <= active_unit.remaining_moves && d > 0 {
    //                         let transform = c.transform.trans(self.j_to_x(j), self.i_to_y(i));
    //                         rectangle(self.reachable_cell_color_mask, reachable_cell, transform, self.gl.as_mut().unwrap());
    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }

    fn render_planned_path(&mut self, c: Context) {
        
        // Check if there is an active unit
        if let Some(active_unit) = self.active_unit.upgrade() {
            
            // If there is an active planned path -> Render it
            if let Some(path) = &self.active_unit_planned_path {

                // Compute cell dimensions in pixel, define reachable mask shape font size and cost text
                let (cell_pix_width, cell_pix_height) = self.cell_pixel();
                let reachable_cell = rectangle_by_corners(0.0, 0.0, cell_pix_width, cell_pix_height);
                let cost_font_size = (cell_pix_height / 6.0).floor() as u32;
                let cost_text = Text {
                    color: [1.0, 1.0, 1.0, 1.0],
                    font_size: cost_font_size,
                    round: false
                };

                let turn_font_size = (cell_pix_height / 4.0).floor() as u32;
                let turn_text = Text {
                    color: [1.0, 0.0, 1.0, 1.0],
                    font_size: turn_font_size,
                    round: false
                };

                let visible_map_bounds = self.visible_map_bounds();
                
                let mut max_cost_in_turn = 0.0;
                for (i, j, cost) in path.iter().skip(1) {
                    if is_in_rect((*i, *j), visible_map_bounds, false) {
                        let (x, y) = self.map_position_to_window_position((*i, *j));
                        let transform = c.transform.trans(x, y);
                        if (*cost - active_unit.borrow().remaining_moves) < 0.0 {
                            max_cost_in_turn = *cost;
                            rectangle([0.0, 1.0, 0.0, 0.3], reachable_cell, transform, self.gl.as_mut().unwrap());
                        }
                        else {
                            let turn_to_arrive = ((*cost - max_cost_in_turn) / active_unit.borrow().speed as f64).ceil();
                            rectangle([1.0, 0.0, 0.0, 0.3], reachable_cell, transform, self.gl.as_mut().unwrap());
                            
                            let turn_to_arrive_str = turn_to_arrive.to_string();
                            let turn_to_arrive_str_len = turn_to_arrive_str.len();
                            
                            let draw_res = turn_text.draw(
                                turn_to_arrive_str.as_str(), self.glyphs.as_mut().unwrap(), 
                                &draw_state::DrawState::default(), 
                                c.transform.trans(x + cell_pix_width - turn_to_arrive_str_len as f64 * turn_font_size as f64, y + cell_pix_height), 
                                self.gl.as_mut().unwrap()
                            );

                            if let Err(_error) = draw_res {
                                dbg!("Something went wrong when drawing planned path !");
                            }
                        }
                        
                        let cost_str = ((cost * 10.0).floor() / 10.0).to_string();
                        let cost_str_len = cost_str.len();
                        
                        let draw_res = cost_text.draw(
                            cost_str.as_str(), self.glyphs.as_mut().unwrap(), 
                            &draw_state::DrawState::default(), 
                            c.transform.trans(x + cell_pix_width - cost_str_len as f64 * cost_font_size as f64, y + cost_font_size as f64), 
                            self.gl.as_mut().unwrap()
                        );

                        if let Err(_error) = draw_res {
                            dbg!("Something went wrong when drawing planned path !");
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
            border: Some(
                graphics::rectangle::Border {
                    color: [0.0, 0.0, 0.0, 1.0],
                    radius: (building_pix_width / 2.0) * border_padding_ratio
                }
            )
        };

        // Draw units
        for i in view_in_map_i1..view_in_map_i2 {
            for j in view_in_map_j1..view_in_map_j2 {
                if let Some(building) = self.building_map.borrow()[(i, j)].upgrade() {
                    building_rectangle
                        .color(self.players[building.borrow().player].principal_color)
                        .border(
                            graphics::rectangle::Border {
                                color: self.players[building.borrow().player].secondary_color,
                                radius: (building_pix_width / 2.0) * border_padding_ratio
                            }
                        )
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
            border: Some(
                graphics::ellipse::Border {
                    color: [1.0, 1.0, 1.0, 1.0],
                    radius: (unit_pix_width / 2.0) * border_padding_ratio
                }
            ),
            resolution: 32
        };

        // Draw units
        for i in view_in_map_i1..view_in_map_i2 {
            for j in view_in_map_j1..view_in_map_j2 {
                if let Some(unit) = self.unit_map.borrow()[(i, j)].upgrade() {
                    let (x, y) = self.map_position_to_window_position((i, j));
                    unit_ellipse
                        .color(self.players[unit.borrow().player].principal_color)
                        .border(
                            graphics::ellipse::Border {
                                color: self.players[unit.borrow().player].secondary_color,
                                radius: (unit_pix_width / 2.0) * border_padding_ratio
                            }
                        )
                        .draw(
                            rectangle, 
                            &draw_state::DrawState::default(), 
                            c.transform.trans(x, y), 
                            self.gl.as_mut().unwrap()
                        );
                }
            }
        }

        // Draw marker on active unit if there is one and if it is visible
        if let Some(active_unit) = self.active_unit.upgrade() {
            if is_in_rect((active_unit.borrow().position.0 as i32, active_unit.borrow().position.1 as i32), visible_map_bounds, false) {
                let cell_padding_ratio = 1.0 / 2.5;
                let marker_pix_width = cell_pix_width * (1.0 - cell_padding_ratio * 2.0);
                let marker_pix_height = cell_pix_height * (1.0 - cell_padding_ratio * 2.0);
                let rectangle = [
                    cell_pix_width * cell_padding_ratio, 
                    cell_pix_height * cell_padding_ratio, 
                    marker_pix_width, 
                    marker_pix_height
                ];
                let active_unit_marker = Ellipse {
                    color: [0.0, 0.0, 0.0, 1.0],
                    border: None,
                    resolution: 32
                };
                let (x, y) = self.map_position_to_window_position(
                    (
                        active_unit.borrow().position.0 as i32, 
                        active_unit.borrow().position.1 as i32
                    )
                );

                active_unit_marker.draw(
                    rectangle, 
                    &draw_state::DrawState::default(), 
                    c.transform.trans(x, y), 
                    self.gl.as_mut().unwrap()
                );
            }
        }
    }

    fn render_territory(&mut self, c: Context) {
        // Compute cell dimensions in pixel
        let (_cell_pix_width, cell_pix_height) = self.cell_pixel();
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

                    let _draw_res = player_text.draw(
                        player.to_string().as_str(), 
                        self.glyphs.as_mut().unwrap(),
                        &draw_state::DrawState::default(), 
                        c.transform.trans(x, y + font_size as f64),
                        self.gl.as_mut().unwrap()
                    );
                }
            }
        }

        // TODO : Implement algorithm to draw territory borders
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
        self.render_planned_path(c);

        // Render units and buildings
        self.render_buildings(c);
        self.render_units(c);

        // Render territory
        self.render_territory(c);

        // End the drawing pipeline
        self.gl.as_mut().unwrap().draw_end();
    }
}
