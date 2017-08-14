extern crate byteorder;
extern crate clap;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate image;
extern crate piston_window;
extern crate rand;

use clap::{App, Arg};
use image::{ImageBuffer, RgbaImage};
use gfx_device_gl::Factory;
use piston_window::*;

use std::fs::File;
use std::io::Read;
use std::time::Instant;

pub mod gameboy;

use gameboy::{Cartridge, Frame, JoypadButton};

fn main() {
    let matches = App::new("gameboy-rs")
        .version("0.0.0.1")
        .author("Simon Whitehead")
        .about("A GameBoy and GameBoy Colour emulator written in Rust")
        .arg(
            Arg::with_name("rom")
                .short("r")
                .long("rom")
                .value_name("ROM_PATH")
                .required(true)
                .help("Path to a Gameboy or Gameboy Color ROM")
                .takes_value(true),
        )
        .get_matches();

    let rom = matches.value_of("rom").unwrap();
    let rom_data = load_rom(rom).unwrap();
    let is_bootrom = rom_data.len() < 0x10A;
    let cart = Cartridge::with_rom(rom_data);
    let mut gameboy = gameboy::GameBoy::new(false, cart, is_bootrom);
    println!("Loading game: {}", gameboy.cart_details().game_title);
    let now = Instant::now();

    let opengl = OpenGL::V3_2;
    let mut window: PistonWindow = WindowSettings::new(
        format!("gbrs: {}", gameboy.cart_details().game_title),
        [160, 144],
    ).exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    window.set_max_fps(60);
    let n = 0;
    'start: while let Some(e) = window.next() {
        if let Some(button) = e.press_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::Return => gameboy.press(JoypadButton::Start),
                    Key::Left => gameboy.press(JoypadButton::Left),
                    Key::Right => gameboy.press(JoypadButton::Right),
                    Key::Space => gameboy.reset(),
                    _ => (),
                }
            }
        }
        if let Some(button) = e.release_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::Return => gameboy.unpress(JoypadButton::Start),
                    Key::Left => gameboy.unpress(JoypadButton::Left),
                    Key::Right => gameboy.unpress(JoypadButton::Right),
                    _ => (),
                }
            }
        }
        let mut factory = window.factory.clone();
        window.draw_2d(&e, |c, g| {
            let img = {
                let frame = gameboy.request_frame();
                build_frame(frame)
            };
            let texture = Texture::from_image(&mut factory, &img, &TextureSettings::new()).unwrap();
            clear([1.0; 4], g);
            image(&texture, c.transform, g);
            if let Err(msg) = gameboy.run() {
                // Dump the last texture we had
                img.save("/Users/Simon/last_frame.png").unwrap();
                panic!(msg);
            }
            Some(())
        });
    }

    // 'init: loop {
    // if Instant::now().duration_since(now) > Duration::new(1, 0) {
    // println!("1 second has elapsed (STUB HERE FOR V-BLANK DEBUGGING)");
    // now = Instant::now();
    // }
    // match  {
    // false => break 'init,
    // _ => (),
    // }
    // }
}

fn load_rom(fname: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(fname)?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    Ok(contents)
}

fn build_frame(frame: &Frame) -> RgbaImage {
    let mut img = ImageBuffer::new(gameboy::SCREEN_WIDTH, gameboy::SCREEN_HEIGHT);
    for x in 0..160 {
        for y in 0..144 {
            let frame_pixel = frame.pixels[160 * y + x];
            let p = image::Rgba([frame_pixel.r, frame_pixel.g, frame_pixel.b, 0xFF]);
            img.put_pixel(x as u32, y as u32, p);
        }
    }

    img
}
