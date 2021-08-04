use std::ops::Sub;
use std::ops::Div;
use rand::prelude::*;
use noise::{NoiseFn, Seedable, Perlin, OpenSimplex, SuperSimplex};
use std::ops::{Index, IndexMut};
use std::collections::VecDeque;

pub trait MapType {}
impl MapType for f64 {}
impl MapType for i32 {}

#[derive(Default)]
pub struct Map<T> {
    pub width: usize,
    pub height: usize,
    pub map: Vec<T>,
}

// impl<T: PartialOrd + Sub<Output = T> + Div<Output = T> + Ord> Map<T> {
//     pub fn normalize(&mut self) {
//         // Normalize map by min-max normalization (Infinity-Norm)
//         let min = *self.map.iter().min().unwrap();
//         let max = *self.map.iter().max().unwrap();

//         self.map = self.map.into_iter().map(|x| (x - min) / (max - min)).collect();
//     }
// }

impl Map<f64> {
    pub fn normalize(&mut self) {
        // Normalize map by min-max normalization (Infinity-Norm)
        let min = self.map.iter().fold(f64::INFINITY, |x, y| x.min(*y));
        let max = self.map.iter().fold(f64::NEG_INFINITY, |x, y| x.max(*y));

        self.map = self.map.iter().map(|x| (*x - min) / (max - min)).collect();
    }
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

pub fn diamond_square(map_size: usize) -> Map<f64> {
    // Initialisation de la carte
    let mut height_map: Map<f64> = Map::<f64>::new(map_size, map_size, 0.0);

    // Initialisation du générateur de nombre aléatoire
    let mut rng = rand::thread_rng();

    // Initialisation des coins
    height_map[(0, 0)] = rng.gen_range((-(map_size as f64))..=(map_size as f64));
    height_map[(0, map_size-1)] = rng.gen_range((-(map_size as f64))..=(map_size as f64));
    height_map[(map_size-1, 0)] = rng.gen_range((-(map_size as f64))..=(map_size as f64));
    height_map[(map_size-1, map_size-1)] = rng.gen_range((-(map_size as f64))..=(map_size as f64));

    let mut diamond_square_queue: VecDeque<(i32, i32, i32)> = VecDeque::new();
    diamond_square_queue.push_back((0, 0, map_size as i32 - 1));
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
                if dpi >= 0 && dpi < map_size as i32 && dpj >= 0 && dpj < map_size as i32 {
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

    // Normalize height map
    height_map.normalize();

    height_map
}

pub fn noise_map(map_size: usize, frequencies: Vec<f64>, frequencies_weight: Vec<f64>, power: f64, island: bool) -> Map<f64> {
    // Initialize noise map
    let mut noise_map: Map<f64> = Map::<f64>::new(map_size, map_size, 0.0);

    // Normalize frequencies weight
    let weights_sum: f64 = frequencies_weight.iter().sum();

    // Initialize random number generator
    let mut rng = rand::thread_rng();

    // Initialize a SuperSimplex
    let mut noise_gen = SuperSimplex::new();

    for (f, w) in frequencies.iter().zip(frequencies_weight.iter()) {
        // Set a different seed for each ferquency
        noise_gen = noise_gen.set_seed(rng.gen());

        // Apply noise to map
        for i in 0..(map_size as i32) {
            for j in 0..(map_size as i32) {
                let ni = (i as f64 / map_size as f64) - 0.5;
                let nj = (j as f64 / map_size as f64) - 0.5;
                noise_map[(i, j)] += w * noise_gen.get([f * ni, f * nj]);
            }
        }
    }

    // Normalize by frequecy weight sum
    noise_map.map = noise_map.map.iter().map(|x| x / weights_sum).collect();

    // Take the power of noise map to make sharper and smoother noise map
    noise_map.map = noise_map.map.iter().map(|x| (*x).powf(power)).collect();

    if island {
        // Apply island transform to map
        let half = (map_size / 2) as f64;
        for i in 0..(map_size as i32) {
            // let wi = 1.0 - ((i as f64 - half).abs() / half);
            for j in 0..(map_size as i32) {
                let ni = ((i as f64 + 0.5) / map_size as f64) - 0.5;
                let nj = ((j as f64 + 0.5) / map_size as f64) - 0.5;
                let d = 2.0 * ni.abs().max(nj.abs());
                let d = (ni.powi(2) + nj.powi(2)).sqrt() / 0.5_f64.sqrt();
                noise_map[(i, j)] = (1.0 + noise_map[(i, j)] - d) / 2.0;
            }
        }
    }

    // Normalize noise map
    noise_map.normalize();

    noise_map
}