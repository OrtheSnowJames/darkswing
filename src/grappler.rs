use raylib::prelude::*;
use crate::object::Object;

const ROPE_LENGTH: f32 = 400.0;
const GRAPPLE_SPEED: f32 = 1200.0;

#[derive(PartialEq, Clone, Copy)]
pub enum GrapplerState {
    Idle,
    Grappling,  // extending
    Retracting, // retracting after miss
    Grappled,   // attached
    Pulling,    // player pulled toward grapple point
}

pub struct Grappler {
    pub position: Vector2,
    pub length: f32,
    pub grapple_point: Vector2,
    pub state: GrapplerState,
    grapple_direction: Vector2,
    pub release_cooldown: f32, // cooldown seconds
}

pub fn point_in_rect(point: Vector2, rect: Rectangle) -> bool {
    point.x >= rect.x &&
    point.x <= rect.x + rect.width &&
    point.y >= rect.y &&
    point.y <= rect.y + rect.height
}

fn get_collision_point(start: Vector2, end: Vector2, objects: &[Object]) -> Option<Vector2> {
    let line_vec = end - start;
    let len = line_vec.length();
    if len == 0.0 {
        return None;
    }
    let dir = line_vec.normalized();
    let step = 5.0; // check every 5 pixels
    let num_steps = (len / step) as i32;

    for object in objects {
        let rect = object.get_rect();
        
        // normal collision detection
        for i in 0..=num_steps {
            let p = start + dir * (i as f32 * step);
            if point_in_rect(p, rect) {
                // snap to corner after hitting object
                return Some(snap_to_corner_if_close(p, rect));
            }
        }
    }
    None
}

fn snap_to_corner_if_close(collision_point: Vector2, rect: Rectangle) -> Vector2 {
    const CORNER_SNAP_DISTANCE: f32 = 20.0; // snap within 20 pixels
    
    let top_left = Vector2::new(rect.x, rect.y);
    let top_right = Vector2::new(rect.x + rect.width, rect.y);
    
    // check collision point close to top-left corner
    if (collision_point - top_left).length() <= CORNER_SNAP_DISTANCE {
        return top_left;
    }
    
    // check collision point close to top-right corner
    if (collision_point - top_right).length() <= CORNER_SNAP_DISTANCE {
        return top_right;
    }
    
    // no corner close enough, return original
    collision_point
}

impl Grappler {
    pub fn new(position: Vector2) -> Self {
        Self {
            position,
            length: 0.0,
            grapple_point: Vector2::zero(),
            state: GrapplerState::Idle,
            grapple_direction: Vector2::zero(),
            release_cooldown: 0.0,
        }
    }

    pub fn draw_with_mouse(&self, dcam: &mut impl RaylibDraw, mouse_position: Vector2) {
        if self.state == GrapplerState::Idle {
            let direction = mouse_position - self.position;
            let normalized_direction = if direction.length_sqr() > 0.0 {
                direction.normalized()
            } else {
                Vector2::new(1.0, 0.0)
            };
            let crosshair_distance = 50.0;
            let crosshair_position = self.position + normalized_direction * crosshair_distance;
            let crosshair_size = 8.0;
            dcam.draw_rectangle_v(
                crosshair_position - Vector2::one() * (crosshair_size / 2.0),
                Vector2::one() * crosshair_size,
                Color::BLACK,
            );
        }

        // draw rope if active
        if self.state != GrapplerState::Idle && self.length > 0.0 {
            let rope_end = if self.state == GrapplerState::Grappled || self.state == GrapplerState::Pulling {
                self.grapple_point
            } else {
                self.position + self.grapple_direction * self.length
            };
            dcam.draw_line_v(self.position, rope_end, Color::BLACK);
        }
    }

    pub fn fire(&mut self, start_pos: Vector2, target_pos: Vector2) {
        if self.state == GrapplerState::Idle {
            self.state = GrapplerState::Grappling;
            self.length = 0.0;
            self.position = start_pos;
            let direction = target_pos - start_pos;
            self.grapple_direction = if direction.length_sqr() > 0.0 {
                direction.normalized()
            } else {
                Vector2::new(1.0, 0.0)
            };
        }
    }

    pub fn update(
        &mut self,
        delta_time: f32,
        objects: &[Object],
    ) {
        match self.state {
            GrapplerState::Grappling => {
                self.length += GRAPPLE_SPEED * delta_time;
                let end_point = self.position + self.grapple_direction * self.length;

                if let Some(collision_point) = get_collision_point(self.position, end_point, objects)
                {
                    self.grapple_point = collision_point;
                    self.length = (self.grapple_point - self.position).length();
                    self.state = GrapplerState::Grappled;
                } else if self.length >= ROPE_LENGTH {
                    self.length = ROPE_LENGTH;
                    self.state = GrapplerState::Retracting;
                }
            }
            GrapplerState::Retracting => {
                self.length -= GRAPPLE_SPEED * delta_time;
                if self.length <= 0.0 {
                    self.length = 0.0;
                    self.state = GrapplerState::Idle;
                    self.release_cooldown = 0.2; // cooldown on miss
                }
            }
            _ => {}
        }
    }
}