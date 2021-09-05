use std::cmp::{Eq, PartialOrd, Ord, Ordering};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use crate::distance::{Distance2D};
use crate::utils::{is_in_rect};
use crate::constraint::{PositionConstraint};

struct CostNode {
    position: (i32, i32),
    cost: f64
}

impl PartialEq for CostNode {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for CostNode {}

impl Ord for CostNode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.cost == other.cost {
            Ordering::Equal
        }
        else if self.cost > other.cost {
            Ordering::Less
        }
        else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for CostNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Const neighbors allowed direction for 4-connexity grid
const NEIGHBORS_DIRECTION_4C: [(i32, i32); 4] = [
    (-1, 0),    // NORTH
    (0, -1),    // WEST
    (1, 0),     // SOUTH
    (0, 1)      // EAST
];

/// Const neighbors allowed direction for 8-connexity grid
const _NEIGHBORS_DIRECTION_8C: [(i32, i32); 8] = [
    (-1, 0),    // NORTH
    (-1, -1),   // NORTH-WEST
    (0, -1),    // WEST
    (-1, 1),    // SOUTH-WEST       
    (1, 0),     // SOUTH
    (1, 1),     // SOUTH-EAST
    (0, 1),     // EAST
    (-1, 1)     // NORTH-EAST
];

/// Function returning all possible neighbors from a position and under certain constraint
/// 
/// # Arguments
/// 
/// * `position` - From which position neighbors search is made
/// * `map_size` - Map size / dimensions used to restrict possible neighbors that go out the map scope
/// * `allowed_directions` - Allowed directions for finding neighbors / Definition of the neighbourhood
/// * `way_position_constraints` - Position constraints that each neighbour must respect to be taken
/// 
pub fn neighbors(
    // Start position from those neighbors will be computed
    position: (i32, i32), 

    // Hypothetic goal
    goal: (i32, i32),

    // Map size / dimensions
    map_size: (i32, i32),

    // Allowed direction to find neighbors
    allowed_directions: &[(i32, i32)],

    // List of position constraints that each neighbour must respect except the goal
    way_position_constraints: &[Box<dyn PositionConstraint>],

    // List of position constraints that each goal neighbour must respect
    goal_position_constraints: &[Box<dyn PositionConstraint>]

) -> Vec<(i32, i32)> {
    
    // Apply neighbors generation and filtering at the same time with functional features
    allowed_directions
        .into_iter()
        .map(|d| (position.0 + d.0, position.1 + d.1))
        .filter(|&x| is_in_rect(x, (0, 0, map_size.0, map_size.1), false))
        .filter(
            |&x| 
            ((x == goal) && (goal_position_constraints.iter().all(|gpc| gpc.respect((x.0 as usize, x.1 as usize))))) ||
            ((x != goal) && (way_position_constraints.iter().all(|pc| pc.respect((x.0 as usize, x.1 as usize)))))
        )
        .collect()
}

/// Utility function to reconstruct path computed by A* algorithm
pub fn reconstruct_path(
    current: (i32, i32), 
    best_previous_node: HashMap<(i32, i32), (i32, i32)>, 
    distance_from_start: HashMap<(i32, i32), f64>
) -> VecDeque<(i32, i32, f64)> {
    let mut total_path = VecDeque::new();
    total_path.push_front((current.0, current.1, *distance_from_start.get(&current).unwrap()));
    let mut current = current;
    while let Some(prev) = best_previous_node.get(&current) {
        current = *prev;
        total_path.push_front((prev.0, prev.1, *distance_from_start.get(&current).unwrap()));
    }

    total_path
}

// TODO : Pass constraints on certains move (position transition/couples)
// TODO : Implement Dijkstra instead of A* for making path finding routine faster, keep A* implementation for complex AI problem as an example of code

/// A* algorithm implementation used to find shortest-path on a 2D map
pub fn astar_2d_map(
    start: (i32, i32), 
    goal: (i32, i32), 
    map_size: (i32, i32), 
    distance: impl Distance2D, 
    heuristic: impl Distance2D,
    way_position_constraints: Vec<Box<dyn PositionConstraint>>,
    goal_position_constraints: Vec<Box<dyn PositionConstraint>>,
) -> Option<VecDeque<(i32, i32, f64)>> {

    // Initialize priority queue as min-binary heap
    // Structure holding potentially next nodes to explore 

    let mut open_priority_queue = BinaryHeap::new();
    let mut open_set = HashSet::new();
    let estimated_path_cost = heuristic.evaluate((start.0 as f64, start.1 as f64), (goal.0 as f64, goal.1 as f64));
    open_priority_queue.push(CostNode{position: start, cost: 0.0});
    open_set.insert(start);

    // For each node, the previous node we have to come from to compose the shortest path from start to this node
    let mut best_previous_node = HashMap::new();

    // For each node, the cost of the cheapest path from start to current node --> best distance score
    let mut distance_from_start = HashMap::new();
    distance_from_start.insert(start, 0.0);

    // For each node, the total cost of the cheapest path from start node to goal passing in current node --> best distance score + best heuritics score
    let mut start_to_goal_cost_by = HashMap::new();
    start_to_goal_cost_by.insert(start, estimated_path_cost);

    while let Some(current) = open_priority_queue.pop() {

        // Define variables to make easier to code
        let current_pos = current.position;
        
        // Check if current node is at goal ==> Reconstruct Path
        if current_pos == goal {
            return Some(reconstruct_path(goal, best_previous_node, distance_from_start));
        }

        open_set.remove(&current.position);

        // Visit each neighbour of current node to explore and find cheapest paths
        for neighbour in &neighbors(current.position, goal, map_size, &NEIGHBORS_DIRECTION_4C, &way_position_constraints, &goal_position_constraints) {
            let current_to_neighbour_distance = distance.evaluate((current_pos.0 as f64, current_pos.1 as f64), (neighbour.0 as f64, neighbour.1 as f64));
            let tentative_distance_from_start = distance_from_start[&current_pos] + current_to_neighbour_distance;

            let mut old_distance_to_start = f64::INFINITY;
            if let Some(distance_from_start) = distance_from_start.get(neighbour) {
                old_distance_to_start = *distance_from_start;
            }

            if tentative_distance_from_start < old_distance_to_start {
                best_previous_node.insert(*neighbour, current_pos);
                distance_from_start.insert(*neighbour, tentative_distance_from_start);

                let cost = tentative_distance_from_start + heuristic.evaluate((neighbour.0 as f64, neighbour.1 as f64), (goal.0 as f64, goal.1 as f64));
                if !open_set.contains(neighbour) {
                    open_set.insert(*neighbour);
                    open_priority_queue.push(CostNode{position: *neighbour, cost: cost});
                }
            }
        }
    }

    None
}
