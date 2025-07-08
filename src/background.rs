use raylib::prelude::*;

pub const TILE_SIZE: f32 = 100.0;
const HALF_WORLD_WIDTH: f32 = 800.0; // covers -800..800 (1600 width)

#[derive(Copy, Clone)]
pub struct Tile {
    pub position: Vector2, // top-left corner
}

impl Tile {
    pub fn draw(&self, d: &mut impl RaylibDraw) {
        // base tile
        d.draw_rectangle(
            self.position.x as i32,
            self.position.y as i32,
            TILE_SIZE as i32,
            TILE_SIZE as i32,
            Color::DARKGRAY,
        );

        let inner_size = (TILE_SIZE * 0.3) as i32; // 30% of tile size
        let offset = ((TILE_SIZE as i32) - inner_size) / 2;
        d.draw_rectangle(
            (self.position.x as i32) + offset,
            (self.position.y as i32) + offset,
            inner_size,
            inner_size,
            Color::new(128, 128, 128, 128), // gray with alpha
        );
    }
}

pub fn generate_tile_layer(_seed: u64, layer_y: i32) -> Vec<Tile> {
    let mut tiles = Vec::<Tile>::new();

    let mut x = -HALF_WORLD_WIDTH;
    while x <= HALF_WORLD_WIDTH {
        tiles.push(Tile { position: Vector2::new(x as f32, layer_y as f32) });
        x += TILE_SIZE as f32;
    }
    tiles
}

pub fn tile_layers_at_y(player_y: f32) -> Vec<i32> {
    const BUFFER: i32 = 1000;

    let mut layers = Vec::new();
    let mut layer_y: i32 = 200;
    while layer_y >= (player_y as i32 - BUFFER) {
        layers.push(layer_y);
        layer_y -= TILE_SIZE as i32;
    }
    let mut layer_y_down: i32 = 300; // first layer below 200 is 300, 400, etc.
    while layer_y_down <= (player_y as i32 + BUFFER) {
        layers.push(layer_y_down);
        layer_y_down += TILE_SIZE as i32;
    }

    layers
} 