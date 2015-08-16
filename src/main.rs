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

use std::cell::RefCell;

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
    let p1_texture = TextureAtlas::from_file(&display, &Path::new("assets/textures/p1_spritesheet.xml")).unwrap();
    let ui_texture = TextureAtlas::from_file(&display, &Path::new("assets/textures/hud_spritesheet.xml")).unwrap();
    println!("atlas: {:?}", p1_texture);


    let mut sprites = SpriteBatch::new(&display, SPRITES_COUNT).unwrap();
    let mut frame_index = 0;
    {
        support::start_loop(|i| {
            // drawing a frame
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 0.0);
            let (vw, vh) = target.get_dimensions();

            sprites.begin(|b|
            {
                match ui_texture.get("coins") {
                    Some(entry) => {
                        let sw = entry.width as f32;
                        let sh = entry.height as f32;
                        b.draw_entry(50.0, 50.0, sw, sh, ui_texture.clone(), entry);
                    },
                    None => {}
                }

            });

            sprites.begin(|b|
            {
                let size = 128;
                let sizef = size as f32;

                let index = ((i / 8) % 11) + 1;
                let frame = format!("p1_walk{:02}", index);
                match p1_texture.get(&frame) {
                    Some(entry) => {
                        //match entry.upgrade() {
                        //    Some(entry) => {
                                if frame_index != index {
                                    println!("drawing sprite: {:?}", entry);
                                    frame_index = index;
                                }
                                let sw = entry.width as f32;
                                let sh = entry.height as f32;
                                b.draw_entry((vw / 2) as f32, (vh / 2) as f32, sw, sh, p1_texture.clone(), entry);
                        //    },
                        //    None => {}
                        //}
                    },
                    None => {}
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
}
