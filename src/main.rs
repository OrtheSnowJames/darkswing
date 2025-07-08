use raylib::prelude::*;
use std::collections::HashMap;
mod object;
mod player;
mod light;
mod grappler;
mod layergen;
mod background;

use object::Object;
use player::{Player, PLAYER_SIZE};
use layergen::{generate_layer, layers_at_y};
use background::{generate_tile_layer, tile_layers_at_y};
use darkswing::{with_drawing, get_position_over_time, circle_radius_for_rect, seeded_random_range, random_range};
const RESPAWN_TIME: f32 = 3.0;
const HOLD_TIME: f32 = 0.5; // 100ms hold after full encapsulation
fn main() {
    let (mut rl, mut thread) = raylib::init()
        .size(800, 600)
        .title("darkswing raylib-rs")
        .resizable()
        .build();

    let seed: u64 = random_range(0.0, 1000000000000000000.0) as u64; // SANS WHO THE F*** THOUGHT THIS WAS A GOOD IDEA

    let mut camera = Camera2D::default();
    camera.offset = Vector2::new(400.0, 300.0); // center of screen
    camera.rotation = 0.0;
    camera.zoom = 1.0;

    // player
    let mut player = Player::new(Vector2::new(100.0, 100.0));

    // respawn timer
    let mut is_respawning = false;
    let mut respawn_timer = 0.0;
    let mut scaled_radius: f32;
    let mut light_texture = rl.load_render_texture(&thread, 800, 600).unwrap();
    let mut screen_texture = rl.load_render_texture(&thread, 800, 600).unwrap();
    let mut light_shader = light::only_on_black_shader(&mut rl, &mut thread);

    let mut objects: Vec<Object> = Vec::new();
    objects.push(Object::new(Vector2::new(50.0, 400.0), Vector2::new(300.0, 50.0))); // ground platform
    objects.push(Object::new(Vector2::new(300.0, 300.0), Vector2::new(150.0, 20.0))); // platform

    let mut layermap: HashMap<i32, Vec<Object>> = HashMap::new();
    let mut tilemap: HashMap<i32, Vec<background::Tile>> = HashMap::new();

    let window_size = Vector2::new(800.0, 600.0);
    let mut actual_window_size: Vector2;

    let light_tex = light::create_light_tex(800, 300.0, &mut rl, &mut thread);
    let flashlight_beam_tex = light::create_flashlight_beam_tex(200, 400, &mut rl, &mut thread);
    let mut darkness_mask = rl.load_render_texture(&mut thread, 800, 600).unwrap();
    let mut darkness_enabled = true; // darkness toggle
    while !rl.window_should_close() {
        let delta_time = rl.get_frame_time();
        actual_window_size = Vector2::new(rl.get_screen_width() as f32, rl.get_screen_height() as f32);

        // darkness toggle
        if rl.is_key_pressed(KeyboardKey::KEY_F) {
            darkness_enabled = !darkness_enabled;
        }

        // coordinate conversion
        // destination rectangle for 800x600 texture to maintain aspect ratio
        let target_aspect = 4.0 / 3.0; // 800x600 aspect ratio
        let window_aspect = actual_window_size.x / actual_window_size.y;
        
        let (dest_width, dest_height) = if window_aspect > target_aspect {
            // window wider than target - fit to height
            let height = actual_window_size.y;
            let width = height * target_aspect;
            (width, height)
        } else {
            // window taller than target - fit to width
            let width = actual_window_size.x;
            let height = width / target_aspect;
            (width, height)
        };
        let x_offset = (actual_window_size.x - dest_width) / 2.0;
        let y_offset = (actual_window_size.y - dest_height) / 2.0;

        // mouse from window-space to texture-space
        let mouse_screen_pos = rl.get_mouse_position();
        let scale_x = window_size.x / dest_width;
        let scale_y = window_size.y / dest_height;
        let mouse_texture_pos = Vector2::new(
            (mouse_screen_pos.x - x_offset) * scale_x,
            (mouse_screen_pos.y - y_offset) * scale_y,
        );

        // texture-space to world-space
        let mouse_position = rl.get_screen_to_world2D(mouse_texture_pos, camera);

        // combine objects for grappler collision detection
        let mut all_objects = objects.clone();
        for layer_objects in layermap.values() {
            all_objects.extend(layer_objects.iter().cloned());
        }
        
        player.input(delta_time, &mut rl, &all_objects, mouse_position);
        
        // update layermap
        let layers = layers_at_y(player.position.y);
        let old_layers = layermap.clone();
        let old_layers = old_layers.keys().collect::<Vec<_>>();
        for layer in old_layers {
            if !layers.contains(layer) {
                layermap.remove(layer);
            }
        }
        for layer in layers {
            if !layermap.contains_key(&layer) {
                layermap.insert(layer, generate_layer(seed, layer));
            }
        }
        
        // update tilemap
        let tile_layers_needed = tile_layers_at_y(player.position.y);
        let old_tile_layers: Vec<i32> = tilemap.keys().cloned().collect();
        for layer in old_tile_layers {
            if !tile_layers_needed.contains(&layer) {
                tilemap.remove(&layer);
            }
        }
        for layer in tile_layers_needed {
            if !tilemap.contains_key(&layer) {
                tilemap.insert(layer, generate_tile_layer(seed, layer));
            }
        }
        
        // player physics
        let update_state = player.update(delta_time, is_respawning);

        if update_state.fell {
            is_respawning = true;
        }
        
        // collisions with objects after player update
        for object in objects.iter() {
            player.update_touch_ground(object);
        }
        
        // collisions with layermap objects
        for layer_objects in layermap.values() {
            for object in layer_objects.iter() {
                player.update_touch_ground(object);
            }
        }
        
        // camera follow player
        camera.target = player.position;
        camera.offset = Vector2::new(window_size.x / 2.0, window_size.y / 2.0); // screen center

        // light texture resize if window resized
        if light_texture.texture.width != window_size.x as i32 || light_texture.texture.height != window_size.y as i32 {
            light_texture = rl.load_render_texture(&thread, window_size.x as u32, window_size.y as u32).unwrap();
        }

        if is_respawning {
            let time_remaining = (RESPAWN_TIME - respawn_timer).max(0.0);
            let min_radius = circle_radius_for_rect(window_size.x, window_size.y);
            let target_radius = PLAYER_SIZE / min_radius; // player size relative to screen
            
            let new_radius = if time_remaining <= 0.0 {
                target_radius  // hold at target radius
            } else {
                get_position_over_time(1.0, target_radius, time_remaining, RESPAWN_TIME)
            };
            
            // respawn + hold time completed
            if respawn_timer >= RESPAWN_TIME + HOLD_TIME {
                player.position = Vector2::new(100.0, 100.0);
                is_respawning = false;
                respawn_timer = 0.0;
            }

            // radius at minimum when shrinking done
            let final_radius = if respawn_timer >= RESPAWN_TIME {
                target_radius  // hold at target size
            } else {
                new_radius
            };
            scaled_radius = final_radius * min_radius;
            
            light::light_with_ring(&mut rl, &mut thread, scaled_radius, window_size, &mut light_texture, &mut light_shader);
        }

        // player screen position for darkness
        let player_screen_pos = rl.get_world_to_screen2D(player.position, camera);
        let light_size = 400.0; // light circle size

        with_drawing(&mut rl.begin_texture_mode(&mut thread, &mut screen_texture), |dtex| {
            with_drawing(&mut dtex.begin_mode2D(camera), |dcam| { // begin mode 2d
                dcam.clear_background(Color::RAYWHITE);

                // background tiles first
                for tile_layer in tilemap.values() {
                    for tile in tile_layer.iter() {
                        tile.draw(dcam);
                    }
                }
                
                player.draw(dcam);
                player.draw_grappler(dcam, mouse_position);

                for object in objects.iter() {
                    object.draw(dcam);
                }
                
                // layermap objects
                for layer_objects in layermap.values() {
                    for object in layer_objects.iter() {
                        object.draw(dcam);
                    }
                }
            });

            if is_respawning {
                dtex.draw_texture_pro(&light_texture, Rectangle::new(0.0, 0.0, light_texture.texture.width as f32, -light_texture.texture.height as f32), Rectangle::new(0.0, 0.0, window_size.x, window_size.y), Vector2::zero(), 0.0, Color::WHITE);
                respawn_timer += delta_time;
            }
        });

        // darkness mask if enabled
        if darkness_enabled {
            with_drawing(&mut rl.begin_texture_mode(&mut thread, &mut darkness_mask), |dtex| {
                dtex.clear_background(Color::BLACK);
                
                // lighting based on equipped tool
                if player.current_tool == player::PlayerTool::Flashlight {
                    // flashlight beam
                    let beam_length = 400.0;
                    let beam_width = 200.0;
                    
                    // rotation angle from flashlight direction
                    let angle = player.flashlight_direction.y.atan2(player.flashlight_direction.x).to_degrees() - 90.0;
                    
                    dtex.draw_texture_pro(&flashlight_beam_tex,
                        Rectangle::new(0.0, 0.0, flashlight_beam_tex.width as f32, flashlight_beam_tex.height as f32),
                        Rectangle::new(player_screen_pos.x, player_screen_pos.y, beam_width, beam_length),
                        Vector2::new(beam_width / 2.0, 0.0), // origin at top center
                        angle,
                        Color::WHITE);
                } else {
                    // circular light
                    dtex.draw_texture_pro(&light_tex, 
                        Rectangle::new(0.0, 0.0, light_tex.width as f32, light_tex.height as f32), 
                        Rectangle::new(player_screen_pos.x - light_size/2.0, player_screen_pos.y - light_size/2.0, light_size, light_size), 
                        Vector2::zero(), 0.0, Color::WHITE);
                }
            });
        }

        with_drawing(&mut rl.begin_drawing(&thread), |d| {
            d.clear_background(Color::BLACK);
            
            d.draw_texture_pro(&screen_texture, 
                Rectangle::new(0.0, 0.0, screen_texture.texture.width as f32, -screen_texture.texture.height as f32), 
                Rectangle::new(x_offset, y_offset, dest_width, dest_height), 
                Vector2::zero(), 0.0, Color::WHITE);

            // darkness effect if enabled
            if darkness_enabled {
                // darkness mask with multiply blending for "hole" effect
                with_drawing(&mut d.begin_blend_mode(BlendMode::BLEND_MULTIPLIED), |d| {
                    d.draw_texture_pro(&darkness_mask, 
                        Rectangle::new(0.0, 0.0, darkness_mask.texture.width as f32, -darkness_mask.texture.height as f32), 
                        Rectangle::new(x_offset, y_offset, dest_width, dest_height), 
                        Vector2::zero(), 0.0, Color::WHITE);
                });
            }
            
            // UI on top of everything
            if !is_respawning {
                let flipped = if player.position.y != 0.0 { -player.position.y } else { 0.0 };
                if flipped > 0.0 {
                    d.draw_text(&format!("Y: {}", flipped.to_string().as_str()), 10, 10, 20, Color::GREEN);
                } else {
                    d.draw_text(&format!("Y not found"), 10, 10, 20, Color::RED);
                }
                
                // tool indicator
                let tool_text = match player.current_tool {
                    player::PlayerTool::Grapple => "Tool: Grapple (1)",
                    player::PlayerTool::Flashlight => "Tool: Flashlight (2)",
                };
                d.draw_text(tool_text, 10, 40, 20, Color::WHITE);
                
                // darkness toggle status
                let darkness_text = if darkness_enabled {
                    "Darkness: ON (F to toggle)"
                } else {
                    "Darkness: OFF (F to toggle)"
                };
                d.draw_text(darkness_text, 10, 70, 20, Color::WHITE);
            }
        });
    }
}
