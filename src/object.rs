use raylib::prelude::*;

const OBJECT_COLOR: Color = Color::RED;

#[derive(Copy, Clone)]
pub struct Object {
    pub position: Vector2,
    pub size: Vector2
}

struct Vector2Int {
    x: i32,
    y: i32
}

impl Vector2Int {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn from_vector2(vector2: Vector2) -> Self {
        Self::new(vector2.x as i32, vector2.y as i32)
    }
}

impl Object {
    pub fn new(position: Vector2, size: Vector2) -> Self {
        Self { position, size }
    }

    pub fn get_rect(&self) -> Rectangle {
        Rectangle::new(self.position.x, self.position.y, self.size.x, self.size.y)
    }

    pub fn draw(&self, d: &mut impl RaylibDraw) {
        let position_int = Vector2Int::from_vector2(self.position);
        let size_int = Vector2Int::from_vector2(self.size);

        d.draw_rectangle(position_int.x, position_int.y, size_int.x, size_int.y, OBJECT_COLOR);
    }
}
