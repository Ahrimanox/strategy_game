
use std::rc::Rc;
use crate::map::Map;
use crate::game::Terrain;

pub trait PositionConstraint {
    fn respect(&self, pos: (usize, usize)) -> bool;
}

pub struct TerrainConstraint {
    pub terrain_map: Rc<Map<Terrain>>,
    pub impractical_terrains: Vec<Terrain>
}

impl PositionConstraint for TerrainConstraint {
    fn respect(&self, pos: (usize, usize)) -> bool {

        // Get underlying terrain with passed position
        let underlying_terrain = &self.terrain_map[pos];

        // Check if underlying terrain is not one of impractical ones
        for impractical_terrain in self.impractical_terrains.iter() {
            if *impractical_terrain == *underlying_terrain {
                return false;
            }
        }

        return true;
    }
}