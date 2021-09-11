
use std::cell::RefCell;
use std::rc::{Weak};
use crate::map::Map;
use crate::game::{Terrain, Unit, Building};

pub trait PositionConstraint {
    fn respect(&self, pos: (usize, usize)) -> bool;
}

pub struct TerrainConstraint {
    pub terrain_map: Weak<RefCell<Map<Weak<Terrain>>>>,
    pub impractical_terrains: Vec<Weak<Terrain>>
}

impl PositionConstraint for TerrainConstraint {
    fn respect(&self, pos: (usize, usize)) -> bool {

        // Get underlying terrain with passed position
        if let Some(terrain_map) = self.terrain_map.upgrade() {
            let underlying_terrain = &terrain_map.borrow()[pos];
            
            // Check if underlying terrain is not one of impractical ones
            if let Some(underlying_terrain) = underlying_terrain.upgrade() {
                for impractical_terrain in self.impractical_terrains.iter() {
                    if let Some(impractical_terrain) = impractical_terrain.upgrade() {
                        if impractical_terrain == underlying_terrain {
                            return false;
                        }
                    }
                }
            }
        }
        
        return true;
    }
}

pub struct UnitConstraint {
    pub unit_map: Weak<RefCell<Map<Weak<RefCell<Unit>>>>>
}

impl PositionConstraint for UnitConstraint {
    fn respect(&self, pos: (usize, usize)) -> bool {

        // Check if there is an underlying unit
        if let Some(unit_map) = self.unit_map.upgrade() {
            return unit_map.borrow()[pos].upgrade().is_none();
        }

        return true;
    }
}

pub struct BuildingConstraint {
    pub building_map: Weak<RefCell<Map<Weak<RefCell<Building>>>>>
}

impl PositionConstraint for BuildingConstraint {
    fn respect(&self, pos: (usize, usize)) -> bool {

        // Check if there is an underlying unit
        if let Some(building_map) = self.building_map.upgrade() {
            return building_map.borrow()[pos].upgrade().is_none();
        }

        return true;
    }
}