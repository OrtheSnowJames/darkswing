// cubes in specific layer (each layer = 100 units)
use crate::object::Object;
use raylib::prelude::*;
use super::seeded_random_range;

pub fn generate_layer(seed: u64, layer_index: i32) -> Vec<Object> {
    // seed + layer for unique stable seed
    let layer_seed = seed ^ (layer_index as u64).wrapping_mul(0x9E3779B97F4A7C15); // golden ratio

    let mut cubes: Vec<Object> = Vec::with_capacity(3);
    for i in 0..3 {
        // different seeds for each cube and axis
        let x_seed = layer_seed.wrapping_mul(13).wrapping_add((i as u64) * 17);
        let y_seed = layer_seed.wrapping_mul(19).wrapping_add((i as u64) * 23);
        
        let x = seeded_random_range(x_seed, -400.0, 400.0);
        let y = layer_index as f32 + seeded_random_range(y_seed, -20.0, 20.0);

        cubes.push(Object::new(Vector2::new(x, y), Vector2::new(30.0, 30.0)));
    }

    cubes
}

pub fn layers_at_y(y: f32) -> Vec<i32> {
    let mut layers = Vec::new();
    
    // layers above player (lower Y) starting from Y=200
    // objects spawn at Y=200, 100, 0, -100, -200, etc. (below Y=300)
    let player_y = y as i32;
    
    // buffer zones so layers don't disappear immediately
    const BUFFER_ABOVE: i32 = 400; // layers 400 units above player
    const BUFFER_BELOW: i32 = 600; // layers 600 units below player
    
    // start Y=200, step upward (decreasing Y) until beyond buffer above player
    let mut layer_y = 200;
    while layer_y >= player_y - BUFFER_ABOVE {
        // layers within buffer window
        if layer_y >= (player_y - BUFFER_ABOVE) && layer_y <= (player_y + BUFFER_BELOW) && layer_y < 300 {
            layers.push(layer_y);
        }
        layer_y -= 200; // move further up (lower Y)
    }

    layers
}

