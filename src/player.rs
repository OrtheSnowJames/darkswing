use raylib::prelude::*;

use crate::object::Object;
use crate::grappler::Grappler;

#[derive(PartialEq, Clone, Copy)]
pub enum PlayerTool {
    Grapple,
    Flashlight,
}

pub const PLAYER_SIZE: f32 = 10.0;
const GRAVITY: f32 = 500.0;

pub struct Player {
    pub position: Vector2,
    pub velocity: Vector2,
    pub grounded: bool,
    pub wall_sliding: bool,
    pub grappler: Grappler,
    pub current_tool: PlayerTool,
    pub flashlight_direction: Vector2,
}

pub struct UpdateState {
    pub fell: bool,
}

impl Player {
    pub fn new(position: Vector2) -> Self {
        Self {
            position,
            velocity: Vector2::zero(),
            grounded: true,
            wall_sliding: false,
            grappler: Grappler::new(position),
            current_tool: PlayerTool::Grapple,
            flashlight_direction: Vector2::zero(),
        }
    }

    pub fn update(&mut self, delta_time: f32, is_respawning: bool) -> UpdateState {
        let mut update_state = UpdateState { fell: false };
        
        self.grappler.position = self.position;
        self.grounded = false;
        self.wall_sliding = false;
        
        if !is_respawning {
            match self.grappler.state {
                crate::grappler::GrapplerState::Pulling => {
                    self.velocity.y += GRAVITY * delta_time;
                    self.position += self.velocity * delta_time;

                    const REEL_IN_SPEED: f32 = 600.0;
                    self.grappler.length = (self.grappler.length - REEL_IN_SPEED * delta_time).max(0.0);

                    self.apply_rope_constraint();

                    if self.grappler.length == 0.0 {
                        self.position = self.grappler.grapple_point;
                        self.velocity = Vector2::zero();
                        self.grappler.state = crate::grappler::GrapplerState::Idle;
                    }

                    self.velocity *= 0.99;
                }
                crate::grappler::GrapplerState::Grappled => {
                    self.velocity.y += GRAVITY * delta_time;
                    self.position += self.velocity * delta_time;

                    self.apply_rope_constraint();

                    self.velocity *= 0.99;
                }
                _ => {
                    self.velocity.y += GRAVITY * delta_time;
                    self.position += self.velocity * delta_time;
                }
            }
        }
        
        if self.has_fallen() {
            update_state.fell = true;
        }

        update_state
    }

    pub fn update_touch_ground(&mut self, enumerated_object: &Object) -> bool {
        // Stop grappling if next frame collides with object
        if self.grappler.state != crate::grappler::GrapplerState::Idle {
            const PREDICT_DT: f32 = 0.05; // 50ms lookahead
            let predicted_pos = self.position + self.velocity * PREDICT_DT;
            let pred_left = predicted_pos.x - PLAYER_SIZE;
            let pred_right = predicted_pos.x + PLAYER_SIZE;
            let pred_top = predicted_pos.y - PLAYER_SIZE;
            let pred_bottom = predicted_pos.y + PLAYER_SIZE;

            let obj_rect = enumerated_object.get_rect();
            if pred_right > obj_rect.x && pred_left < obj_rect.x + obj_rect.width &&
               pred_bottom > obj_rect.y && pred_top < obj_rect.y + obj_rect.height {
                self.grappler.state = crate::grappler::GrapplerState::Idle;
                self.grappler.length = 0.0;
                self.grappler.release_cooldown = 0.1; // brief cooldown
            }
        }

        let horizontal_expansion: f32 = if self.grappler.state == crate::grappler::GrapplerState::Grappled ||
            self.grappler.state == crate::grappler::GrapplerState::Pulling {
            2.0 
        } else {
            10.0
        };
        const SLIDE_SPEED: f32 = 50.0;
        
        let expanded_width = enumerated_object.size.x + (horizontal_expansion * 2.0);
        let object_left = enumerated_object.position.x - horizontal_expansion;
        let object_right = object_left + expanded_width;
        let object_top = enumerated_object.position.y;
        let object_bottom = enumerated_object.position.y + enumerated_object.size.y;
        
        let player_left = self.position.x - PLAYER_SIZE;
        let player_right = self.position.x + PLAYER_SIZE;
        let player_top = self.position.y - PLAYER_SIZE;
        let player_bottom = self.position.y + PLAYER_SIZE;
        
        if player_right > object_left && player_left < object_right &&
           player_bottom > object_top && player_top < object_bottom {
            if self.grappler.state == crate::grappler::GrapplerState::Grappled &&
               crate::grappler::point_in_rect(self.grappler.grapple_point, enumerated_object.get_rect()) {
                self.grappler.state = crate::grappler::GrapplerState::Pulling;
            }
            
            let orig_object_left = enumerated_object.position.x;
            let orig_object_right = enumerated_object.position.x + enumerated_object.size.x;
            
            let overlap_left = player_right - orig_object_left;
            let overlap_right = orig_object_right - player_left;
            let overlap_top = player_bottom - object_top;
            let overlap_bottom = object_bottom - player_top;
            
            let min_overlap = overlap_left.min(overlap_right).min(overlap_top).min(overlap_bottom);
            
            if self.grappler.state == crate::grappler::GrapplerState::Grappled || 
               self.grappler.state == crate::grappler::GrapplerState::Pulling {
                
                if min_overlap == overlap_top {
                    self.position.y = object_top - PLAYER_SIZE;
                    if self.velocity.y > 0.0 {
                        self.velocity.y = 0.0;
                    }
                    self.grounded = true;
                    return true;
                } else if min_overlap == overlap_bottom {
                    self.position.y = object_bottom + PLAYER_SIZE;
                    if self.velocity.y < 0.0 {
                        self.velocity.y = 0.0;
                    }
                } else if min_overlap == overlap_left {
                    self.position.x = orig_object_left - PLAYER_SIZE;
                    if self.velocity.x > 0.0 {
                        self.velocity.x = 0.0;
                    }
                    self.wall_sliding = true;
                } else if min_overlap == overlap_right {
                    self.position.x = orig_object_right + PLAYER_SIZE;
                    if self.velocity.x < 0.0 {
                        self.velocity.x = 0.0;
                    }
                    self.wall_sliding = true;
                }
                return false;
            }
            
            if min_overlap == overlap_top && self.velocity.y > 0.0 {
                self.position.y = object_top - PLAYER_SIZE;
                self.velocity.y = 0.0;
                self.grounded = true;
                return true;
            } else if min_overlap == overlap_bottom && self.velocity.y < 0.0 {
                self.position.y = object_bottom + PLAYER_SIZE;
                self.velocity.y = SLIDE_SPEED;
                self.wall_sliding = true;
            } else if min_overlap == overlap_left && self.velocity.x > 0.0 {
                self.position.x = orig_object_left - PLAYER_SIZE;
                self.velocity.x = 0.0;
                self.velocity.y = SLIDE_SPEED;
                self.wall_sliding = true;
            } else if min_overlap == overlap_right && self.velocity.x < 0.0 {
                self.position.x = orig_object_right + PLAYER_SIZE;
                self.velocity.x = 0.0;
                self.velocity.y = SLIDE_SPEED;
                self.wall_sliding = true;
            }
        }
        
        false
    }

    pub fn input(&mut self, delta_time: f32, rl: &mut RaylibHandle, objects: &[Object], mouse_pos: Vector2) {
        let prev_grapple_state = self.grappler.state;
        
        // Tool switching
        if rl.is_key_pressed(KeyboardKey::KEY_ONE) {
            self.current_tool = PlayerTool::Grapple;
        } else if rl.is_key_pressed(KeyboardKey::KEY_TWO) {
            self.current_tool = PlayerTool::Flashlight;
            // Release grapple on tool switch
            self.grappler.state = crate::grappler::GrapplerState::Idle;
        }
        
        // Flashlight direction toward mouse
        if self.current_tool == PlayerTool::Flashlight {
            let direction = mouse_pos - self.position;
            self.flashlight_direction = if direction.length_sqr() > 0.0 {
                direction.normalized()
            } else {
                Vector2::new(1.0, 0.0)
            };
        }
        
        // Grapple fire/release
        if self.current_tool == PlayerTool::Grapple && self.grappler.release_cooldown <= 0.0 {
            if rl.is_key_pressed(KeyboardKey::KEY_E) || rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
                match self.grappler.state {
                    crate::grappler::GrapplerState::Idle => self.grappler.fire(self.position, mouse_pos),
                    crate::grappler::GrapplerState::Grappled => self.grappler.state = crate::grappler::GrapplerState::Pulling,
                    crate::grappler::GrapplerState::Pulling => {
                        self.grappler.state = crate::grappler::GrapplerState::Idle;
                        self.grappler.release_cooldown = 0.2; // cooldown on release
                    },
                    _ => {},
                }
            }
        }

        // Grappler update
        self.grappler.update(delta_time, objects);
        if self.grappler.release_cooldown > 0.0 {
            self.grappler.release_cooldown -= delta_time;
        }

        // Reset velocity on grapple release
        if prev_grapple_state != crate::grappler::GrapplerState::Idle && self.grappler.state == crate::grappler::GrapplerState::Idle {
            self.velocity = Vector2::zero();
        }
        
        // Jumping
        if rl.is_key_down(KeyboardKey::KEY_SPACE) ||
           rl.is_key_down(KeyboardKey::KEY_W) ||
           rl.is_key_down(KeyboardKey::KEY_UP) {
            
            if self.grounded {
                self.velocity.y = -300.0; // normal jump
            } else if self.wall_sliding {
                self.velocity.y = -300.0; // wall climb boost
            }
        }

        if rl.is_key_down(KeyboardKey::KEY_LEFT) || rl.is_key_down(KeyboardKey::KEY_A) {
            self.velocity.x = -150.0;
        } else if rl.is_key_down(KeyboardKey::KEY_RIGHT) || rl.is_key_down(KeyboardKey::KEY_D) {
            self.velocity.x = 150.0;
        } else {
            self.velocity.x *= 0.8; 
        }
        
        // Fast fall
        if (rl.is_key_down(KeyboardKey::KEY_S) ||
            rl.is_key_down(KeyboardKey::KEY_DOWN)) &&
            !self.grounded {
            self.velocity.y += 200.0 * delta_time;
        }
    }

    pub fn has_fallen(&self) -> bool {
        self.position.y > 1000.0
    }
    
    pub fn draw(&self, d: &mut impl RaylibDraw) {
        d.draw_circle(self.position.x as i32, self.position.y as i32, PLAYER_SIZE, Color::BLUE);
    }
    
    pub fn draw_grappler(&self, d: &mut impl RaylibDraw, mouse_position: Vector2) {
        if self.current_tool == PlayerTool::Grapple {
            self.grappler.draw_with_mouse(d, mouse_position);
        } else if self.current_tool == PlayerTool::Flashlight {
            let flashlight_distance = 50.0;
            let crosshair_position = self.position + self.flashlight_direction * flashlight_distance;
            let crosshair_size = 6.0;
            d.draw_rectangle_v(
                crosshair_position - Vector2::one() * (crosshair_size / 2.0),
                Vector2::one() * crosshair_size,
                Color::YELLOW,
            );
        }
    }


    fn apply_rope_constraint(&mut self) {
        let rope_vec = self.position - self.grappler.grapple_point;
        let dist = rope_vec.length();
        if dist > 0.0 {
            let rope_dir = rope_vec / dist;
            let desired_position = self.grappler.grapple_point + rope_dir * self.grappler.length;

            self.position = desired_position;

            let radial_speed = self.velocity.dot(rope_dir);
            self.velocity -= rope_dir * radial_speed;
        }
    }
}
