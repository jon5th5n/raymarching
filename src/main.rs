use std::f32::consts::PI;

use macroquad::{prelude::*, window};

mod instruction_builder;
use crate::instruction_builder::*;
mod rm_camera;
use crate::rm_camera::*;

#[macroquad::main("raymarching")]
async fn main() {
    let fragment_shader = std::fs::read_to_string("./src/shader.fs").unwrap();
    let vertex_shader = std::fs::read_to_string("./src/shader.vs").unwrap();

    let mut material = load_material(
        &vertex_shader,
        &fragment_shader,
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                ..Default::default()
            },
            uniforms: vec![("test".to_string(), UniformType::Float1)],
            textures: vec!["./tex".to_string()],
            ..Default::default()
        },
    )
    .unwrap();

    // let t0 = 0b01010101001100110000111101010101u32;
    // let t1 = 0b01010101u32;
    // let t2 = 0b00110011u32;
    // let t3 = 0b00001111u32;
    // let t4 = 0b01010101u32;
    // println!("{:032b}", ((t1 << 24) + (t2 << 16) + (t3 << 8) + (t4)));
    // println!("{:032b}", (t0 & 0b11111111000000000000000000000000));
    // println!("{:032b}", (t0 >> 24));
    // println!("{:032b}", (t0 >> 16) & 0b00000000000000000000000011111111);
    // println!("{:032b}", (t0 >> 8) & 0b00000000000000000000000011111111);
    // println!("{:032b}", (t0 & 0b00000000000000000000000011111111));

    // let f0 = 1.0f32;

    let mut bytes =
        build_instruction_bytes(&std::fs::read_to_string("./instructions.txt").unwrap());

    let unfinished_u32s = bytes.len() % 4;
    if unfinished_u32s != 0 {
        bytes.append(&mut vec![0; 4 - unfinished_u32s]);
    }

    let tex = Texture2D::from_rgba8((bytes.len() / 4) as u16, 1, &bytes);

    let mut rm_camera = RMCamera::new(
        (window::screen_width() / 10.0) as u32,
        (window::screen_height() / 10.0) as u32,
        PI / 2.0,
        500.0,
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
    );

    loop {
        println!("{}", get_fps());

        draw_shader(&mut material, &rm_camera, tex);

        if is_key_down(KeyCode::W) {
            rm_camera.move_forward(1.0);
        }
        if is_key_down(KeyCode::S) {
            rm_camera.move_forward(-1.0);
        }
        if is_key_down(KeyCode::A) {
            rm_camera.move_right(-1.0);
        }
        if is_key_down(KeyCode::D) {
            rm_camera.move_right(1.0);
        }
        if is_key_down(KeyCode::Space) {
            rm_camera.move_up(1.0);
        }
        if is_key_down(KeyCode::LeftShift) {
            rm_camera.move_up(-1.0);
        }

        if is_key_down(KeyCode::Up) {
            rm_camera.rotate_vertical(PI / 180.0);
        }
        if is_key_down(KeyCode::Down) {
            rm_camera.rotate_vertical(-PI / 180.0);
        }
        if is_key_down(KeyCode::Right) {
            rm_camera.rotate_horizontal(-PI / 180.0);
        }
        if is_key_down(KeyCode::Left) {
            rm_camera.rotate_horizontal(PI / 180.0);
        }

        rm_camera.set_width((window::screen_width() / 10.0) as u32);
        rm_camera.set_height((window::screen_height() / 10.0) as u32);

        next_frame().await
    }
}

fn draw_shader(material: &mut Material, camera: &RMCamera, passed_data: Texture2D) {
    let fragment_shader = std::fs::read_to_string("./src/shader.fs").unwrap();
    let vertex_shader = std::fs::read_to_string("./src/shader.vs").unwrap();

    material.delete();
    *material = load_material(
        &vertex_shader,
        &fragment_shader,
        MaterialParams {
            pipeline_params: PipelineParams {
                depth_write: true,
                depth_test: Comparison::LessOrEqual,
                ..Default::default()
            },
            uniforms: vec![
                ("cam_size".to_string(), UniformType::Int2),
                ("cam_fov".to_string(), UniformType::Float1),
                ("cam_depth".to_string(), UniformType::Float1),
                ("cam_position".to_string(), UniformType::Float3),
                ("cam_direction".to_string(), UniformType::Float3),
                ("cam_up".to_string(), UniformType::Float3),
                ("cam_right".to_string(), UniformType::Float3),
            ],
            textures: vec!["./tex".to_string()],
            ..Default::default()
        },
    )
    .unwrap();

    material.set_uniform(
        "cam_size",
        IVec2::new(camera.get_width() as i32, camera.get_height() as i32),
    );
    material.set_uniform("cam_fov", camera.get_fov());
    material.set_uniform("cam_depth", camera.get_depth());
    material.set_uniform("cam_position", camera.get_position());
    material.set_uniform("cam_direction", camera.get_direction());
    material.set_uniform("cam_up", camera.get_up());
    material.set_uniform("cam_right", camera.get_right());

    clear_background(WHITE);
    set_default_camera();
    gl_use_material(*material);

    draw_texture_ex(
        passed_data,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(Vec2 {
                x: window::screen_width(),
                y: window::screen_height(),
            }),
            ..Default::default()
        },
    );
}
