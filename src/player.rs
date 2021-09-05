use std::cell::{RefCell};
use std::rc::{Rc};

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Unit {

    // Unit player
    pub player: usize,

    // Unit position in map
    pub position: (usize, usize),

    // Attributes
    pub damage: f64,
    pub health: f64,
    pub speed: f64,
    pub remaining_moves: f64
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Building {

    // Building player
    pub player: usize,

    // Building position in map
    pub position: (usize, usize),

    // Attributes
    pub damage: i32,
    pub health: i32,
}

pub struct Player {

    // Player num/code
    pub num: usize,

    // Buildings position
    pub buildings: Vec<Rc<RefCell<Building>>>,

    // Units position
    pub units: Vec<Rc<RefCell<Unit>>>,

    // Colors
    pub principal_color: [f32; 4],
    pub secondary_color: [f32; 4]
}

impl Player {

    pub fn new(
        num: usize, 
        base_position: (usize, usize), 
        principal_color: [f32; 4], 
        secondary_color: [f32; 4]
    ) -> Player {

        let mut player = Player {
            num: num,
            buildings: Vec::new(),
            units: Vec::new(),
            principal_color: principal_color,
            secondary_color: secondary_color
        };

        player.buildings.push(
            Rc::new(
                RefCell::new(
                    Building {
                        player: num,
                        position: base_position,
                        damage: 0,
                        health: 1
                    }
                )
            )
        );

        return player
    }

    // TODO : Add a function to create new units in a neighbourhood of generator building
}