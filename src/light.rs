use raylib::prelude::*;

pub fn create_light_tex(size: i32, max_radius: f32, rl: &mut RaylibHandle, rl_thread: &mut RaylibThread) -> Texture2D {
    let mut light_img = Image::gen_image_color(size, size, Color::new(0, 0, 0, 0));
    
    let center = size as f32 / 2.0;
    
    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let distance = (dx * dx + dy * dy).sqrt();
            
            if distance <= max_radius {
                let alpha_factor = distance / max_radius;
                let alpha_factor = alpha_factor * alpha_factor;
                let alpha = (255.0 * (1.0 - alpha_factor)) as u8;
                
                let pixel_color = Color::new(255, 255, 255, alpha);
                light_img.draw_pixel(x, y, pixel_color);
            }
        }
    }
    
    let light_tex = rl.load_texture_from_image(rl_thread, &mut light_img);
    drop(light_img);
    light_tex.unwrap()
}

pub fn create_flashlight_beam_tex(width: i32, height: i32, rl: &mut RaylibHandle, rl_thread: &mut RaylibThread) -> Texture2D {
    let mut beam_img = Image::gen_image_color(width, height, Color::new(0, 0, 0, 0));
    
    let center_x = width as f32 / 2.0;
    let beam_angle = 30.0; // beam angle degrees
    let beam_length = height as f32;
    
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - center_x;
            let dy = y as f32;
            
            // angle from center line
            let angle = (dx / dy).atan().to_degrees();
            
            // point within beam cone
            if dy > 0.0 && angle.abs() <= beam_angle / 2.0 {
                // distance from center line
                let distance_from_center = dx.abs();
                let max_width_at_distance = dy * (beam_angle / 2.0).to_radians().tan();
                
                if distance_from_center <= max_width_at_distance {
                    // alpha based on distance from center and source
                    let center_factor = 1.0 - (distance_from_center / max_width_at_distance);
                    let distance_factor = 1.0 - (dy / beam_length);
                    let alpha = (255.0 * center_factor * distance_factor * 0.8) as u8;
                    
                    let pixel_color = Color::new(255, 255, 255, alpha);
                    beam_img.draw_pixel(x, y, pixel_color);
                }
            }
        }
    }
    
    let beam_tex = rl.load_texture_from_image(rl_thread, &mut beam_img);
    drop(beam_img);
    beam_tex.unwrap()
}

pub fn light_with_ring(rl: &mut RaylibHandle, rl_thread: &mut RaylibThread, radius: f32, size: Vector2, render_texture: &mut RenderTexture2D, shader: &mut Shader) {
    
    // draw to it
    let mut text_mode = rl.begin_texture_mode(rl_thread, render_texture);
    let blnd = BlendMode::BLEND_SUBTRACT_COLORS;

    text_mode.clear_background(Color::BLACK);

    let mut blnd_mode = text_mode.begin_blend_mode(blnd);
    
    blnd_mode.draw_circle((size.x / 2.) as i32, (size.y / 2.) as i32, radius, Color::WHITE);

    drop(blnd_mode);

    let mut shader_mode = text_mode.begin_shader_mode(shader);
    shader_mode.draw_circle((size.x / 2.) as i32, (size.y / 2.) as i32, radius, Color::WHITE);
    
    // large centered respawn text
    let text = "RESPAWNING...";
    let font_size = 60;
    let text_width = shader_mode.measure_text(text, font_size);
    let text_x = (size.x / 2.0) - (text_width as f32 / 2.0);
    let text_y = (size.y / 2.0) - (font_size as f32 / 2.0);
    shader_mode.draw_text(text, text_x as i32, text_y as i32, font_size, Color::WHITE);

    drop(shader_mode);
    drop(text_mode); // avoid borrowing    
}

const ONLY_ON_BLACK_SHADER: &str = r#"
#version 330

in vec2 fragTexCoord;
uniform sampler2D texture0;
uniform sampler2D background;

out vec4 finalColor;

void main() {
    vec4 bg = texture(background, fragTexCoord);
    vec4 txt = texture(texture0, fragTexCoord);

    if (bg.rgb == vec3(0.0)) {
        finalColor = txt;
    } else {
        discard;
    }
}
"#;

pub fn only_on_black_shader(rl: &mut RaylibHandle, rl_thread: &mut RaylibThread) -> Shader {
    std::fs::write("/tmp/shader.glsl", ONLY_ON_BLACK_SHADER).unwrap();
    let shader = rl.load_shader(rl_thread, None, Some("/tmp/shader.glsl"));
    std::fs::remove_file("/tmp/shader.glsl").unwrap();
    shader
}