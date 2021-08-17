use std::cmp::{Eq, PartialOrd, Ord, Ordering};
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};

use crate::distance::{Distance2D};

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

pub fn neighbours_4c(position: (i32, i32), map_size: (i32, i32)) -> Vec<(i32, i32)> {
    
    let mut neighbours = Vec::new();

    let is_at_west = position.0 == 0;
    let is_at_east = position.0 == (map_size.0 - 1);
    let is_at_north = position.1 == 0;
    let is_at_south = position.1 == (map_size.1 - 1);

    if !is_at_west {neighbours.push((position.0 - 1, position.1));}
    if !is_at_east {neighbours.push((position.0 + 1, position.1));}
    if !is_at_north {neighbours.push((position.0, position.1 - 1));}
    if !is_at_south {neighbours.push((position.0, position.1 + 1));}

    neighbours
}

pub fn neighbours_8c(position: (i32, i32), map_size: (i32, i32)) -> Vec<(i32, i32)> {
    
    let mut neighbours = Vec::new();

    let is_at_west = position.0 == 0;
    let is_at_east = position.0 == (map_size.0 - 1);
    let is_at_north = position.1 == 0;
    let is_at_south = position.1 == (map_size.1 - 1);

    if !is_at_west {neighbours.push((position.0 - 1, position.1));}
    if !is_at_east {neighbours.push((position.0 + 1, position.1));}
    if !is_at_north {neighbours.push((position.0, position.1 - 1));}
    if !is_at_south {neighbours.push((position.0, position.1 + 1));}

    if !is_at_west && !is_at_north {neighbours.push((position.0 - 1, position.1 - 1));}
    if !is_at_west && !is_at_south {neighbours.push((position.0 - 1, position.1 + 1));}
    if !is_at_east && !is_at_north {neighbours.push((position.0 + 1, position.1 - 1));}
    if !is_at_east && !is_at_south {neighbours.push((position.0 + 1, position.1 + 1));}

    neighbours
}

pub fn reconstruct_path(current: (i32, i32), best_previous_node: HashMap<(i32, i32), (i32, i32)>, distance_from_start: HashMap<(i32, i32), f64>) -> VecDeque<(i32, i32, f64)> {
    let mut total_path = VecDeque::new();
    total_path.push_front((current.0, current.1, *distance_from_start.get(&current).unwrap()));
    let mut current = current;
    while let Some(prev) = best_previous_node.get(&current) {
        current = *prev;
        total_path.push_front((prev.0, prev.1, *distance_from_start.get(&current).unwrap()));
    }

    total_path
}


pub fn astar_2d_map(start: (i32, i32), goal: (i32, i32), map_size: (i32, i32), distance: impl Distance2D, heuristic: impl Distance2D) -> Option<VecDeque<(i32, i32, f64)>> {
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
        for neighbour in &neighbours_4c(current.position, map_size) {
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
