pub mod player;
pub mod object;
pub mod light;
pub mod grappler;
pub mod layergen;

use raylib::prelude::*;
use rand::{rng, Rng};
use rand::{SeedableRng};
use rand_chacha::ChaCha8Rng;

pub fn with_drawing<T, F>(draw: &mut T, f: F)
where
    T: RaylibDraw,
    F: FnOnce(&mut T),
{
    f(draw);
}

pub fn get_position_over_time(start: f32, end: f32, time_remaining: f32, total_time: f32) -> f32 {
    
    let t = ((total_time - time_remaining) / total_time).clamp(0.0, 1.0);
    start + (end - start) * t
}

pub fn circle_radius_for_rect(w: f32, h: f32) -> f32 {
    ((w * w) + (h * h)).sqrt() / 2.0
}

pub fn find_aspect_ratio(window_size: Vector2) -> Vector2 {
    if window_size.x == 800.0 && window_size.y == 600.0 {
        return Vector2::new(4.0, 3.0);
    }
    // simplify x/y
    let gcd = gcd(window_size.x, window_size.y);
    Vector2::new(window_size.x / gcd, window_size.y / gcd)
}

fn gcd(a: f32, b: f32) -> f32 {
    if b == 0.0 {
        a
    } else {
        gcd(b, a % b)
    }
}

pub fn random_vec<T: Clone>(vect: &[T]) -> T {
    let mut rng = rng();
    let idx = rng.random_range(0..vect.len());
    vect[idx].clone()
}

pub fn random_range(min: f32, max: f32) -> f32 {
    let mut rng = rng();
    rng.random_range(min..max)
}

pub fn seeded_random_range(seed: u64, min: f32, max: f32) -> f32 {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    rng.random_range(min..max)
}

pub fn seeded_random_vec<T: Clone>(seed: u64, vect: &[T]) -> T {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let idx = rng.random_range(0..vect.len());
    vect[idx].clone()
}