use rand::prelude::*;
use std::ops::{Index, IndexMut};
use std::collections::VecDeque;

#[derive(Default)]
pub struct Map<T> {
    pub width: usize,
    pub height: usize,
    pub map: Vec<T>,
}

impl<T: Clone> Map<T> {
    pub fn new(width: usize, height: usize, value: T) -> Map<T> {
        // Instantiate underlying map vector and fill it with a predefined
        let mut map = Vec::new();
        map.resize(width * height, value);

        Map::<T> {
            width: width,
            height: height,
            map
        }
    }
}

impl<T> Index<(usize, usize)> for Map<T> {
    type Output = T;
    fn index(&self, pos: (usize, usize)) -> &Self::Output {
        let (i, j) = pos;
        &self.map[i * self.width + j]
    }
}

impl<T> IndexMut<(usize, usize)> for Map<T> {
    fn index_mut(&mut self, pos: (usize, usize)) -> &mut Self::Output {
        let (i, j) = pos;
        &mut self.map[i * self.width + j]
    }
}

impl<T> Index<(i32, i32)> for Map<T> {
    type Output = T;
    fn index(&self, pos: (i32, i32)) -> &Self::Output {
        let (i, j) = pos;
        &self.map[i as usize * self.width + j as usize]
    }
}

impl<T> IndexMut<(i32, i32)> for Map<T> {
    fn index_mut(&mut self, pos: (i32, i32)) -> &mut Self::Output {
        let (i, j) = pos;
        &mut self.map[i as usize * self.width + j as usize]
    }
}

pub fn diamond_square(n: u32, normalize: bool) -> Map<f64> {
    let base: usize = 2;
    let map_dim = base.pow(4+n)+1;

    // Initialisation de la carte
    let mut height_map: Map<f64> = Map::<f64>::new(map_dim, map_dim, 0.0);

    // Initialisation du générateur de nombre aléatoire
    let mut rng = rand::thread_rng();

    // Initialisation des coins
    height_map[(0, 0)] = rng.gen_range((-(map_dim as f64))..=(map_dim as f64));
    height_map[(0, map_dim-1)] = rng.gen_range((-(map_dim as f64))..=(map_dim as f64));
    height_map[(map_dim-1, 0)] = rng.gen_range((-(map_dim as f64))..=(map_dim as f64));
    height_map[(map_dim-1, map_dim-1)] = rng.gen_range((-(map_dim as f64))..=(map_dim as f64));

    let mut diamond_square_queue: VecDeque<(i32, i32, i32)> = VecDeque::new();
    diamond_square_queue.push_back((0, 0, map_dim as i32 - 1));
    let end_step = 2;
    while let Some((i, j, s)) = diamond_square_queue.pop_front() {
        
        // Etape du Diamant
        let rand_add = rng.gen_range((-(s as f64))..=(s as f64));
        let mean = (height_map[(i as usize, j as usize)] + height_map[(i as usize, (j+s) as usize)] + height_map[((i+s) as usize, j as usize)] + height_map[((i+s) as usize, (j+s) as usize)]) / 4.0;
        height_map[((i+s/2) as usize, (j+s/2) as usize)] = mean + rand_add;

        // Etape du Carre
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
                    value_sum += height_map[(dpi as usize, dpj as usize)];
                }
            }

            let rand_add = rng.gen_range((-(s as f64))..=(s as f64));
            height_map[(pi as usize, pj as usize)] = ((value_sum as f64) / (value_num as f64)) + rand_add;
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
        let mut min = 100.0 * map_dim as f64;
        let mut max = -100.0 * map_dim as f64;
        for h in height_map.map.iter() {
            if *h < min {min = *h;}
            if *h > max {max = *h;}
        }

        for h in height_map.map.iter_mut() {
            *h = (*h - min) / (max - min);
        }
    }

    height_map
}