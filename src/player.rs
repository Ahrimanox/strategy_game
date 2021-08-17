#[derive(Debug, PartialEq, Copy, Clone)]
pub struct Unit {
    // Unit player
    pub player: usize,

    // Unit position in map
    pub position: [usize; 2],

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
    pub position: [usize; 2],

    // Attributes
    pub damage: i32,
    pub health: i32,
}

pub struct Player {
    // Base position
    pub base_position: [usize; 2],

    // Buildings position
    pub buildings_position: Vec<[usize; 2]>,

    // Units position
    pub units_position: Vec<[usize; 2]>,

    // Colors
    pub principal_color: [f32; 4],
    pub secondary_color: [f32; 4]
}

impl Player {
    pub fn new(base_position: [usize; 2], principal_color: [f32; 4], secondary_color: [f32; 4]) -> Player {
        Player {
            base_position: base_position,
            buildings_position: Vec::<[usize; 2]>::new(),
            units_position: Vec::<[usize; 2]>::new(),
            principal_color: principal_color,
            secondary_color: secondary_color
        }
    }
}