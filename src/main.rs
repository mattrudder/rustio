#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]

#[macro_use]
extern crate glium;
extern crate rand;

use std::path::Path;
use glium::{DisplayBuild, Surface};
use glium::index::PrimitiveType;
use glium::glutin;

mod support;
mod sprites;
use sprites::{TextureAtlas, SpriteBatch};

const SPRITES_COUNT: usize = 1024;

fn main() {
    println!("Number of sprites: {}", SPRITES_COUNT);

    let display = glutin::WindowBuilder::new()
        .with_dimensions(1280, 720)
        .with_title(format!("Rustio"))
        .with_gl(glutin::GlRequest::Latest)
        .with_gl_profile(glutin::GlProfile::Core)
        .build_glium().unwrap();

    // the main loop
    println!("current dir: {:?}", std::env::current_dir());
    let atlas = TextureAtlas::from_file(&display, &Path::new("assets/textures/p1_spritesheet.xml")).unwrap();
    println!("atlas: {:?}", atlas);

    let mut sprites = SpriteBatch::new(&display, SPRITES_COUNT).unwrap();
    support::start_loop(|| {
        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);

        // TODO: Fix lifetime issue with target. Might need to make sprites.begin callback
        // something that does all the sprite drawing?
        sprites.begin(|b|
        {
            for y in 0..1 {
                for x in 0..1 {
                    //let pos: (f32, f32) = (rand::random(), rand::random());
                    println!("batching sprite at {},{}", x, y);
                    b.draw(x as f32, y as f32, 32.0, 32.0, atlas.clone());
                }
            }
        });

        sprites.end(&mut target);
        target.finish().unwrap();

        for event in display.poll_events() {
            match event {
                glutin::Event::Closed => return support::Action::Stop,
                _ => ()
            }
        }

        support::Action::Continue
    });
}
