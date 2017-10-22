extern crate byteorder;
extern crate clap;
#[macro_use]
extern crate conrod;
extern crate find_folder;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate graphics;
extern crate image;
extern crate piston_window;
extern crate rand;

use clap::{App, Arg};
use image::{ImageBuffer, RgbaImage};
use gfx_device_gl::Factory;
use piston_window::*;
use piston_window::OpenGL;

use graphics::draw_state::Blend;

use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::thread;
use std::time::Instant;

pub mod gameboy;

use gameboy::{Cartridge, CpuSpeed, Frame, JoypadButton, Ui};

fn main() {
    let matches = App::new("gameboy-rs")
        .version("0.0.0.1")
        .author("Simon Whitehead")
        .about("A GameBoy and GameBoy Colour emulator written in Rust")
        .arg(Arg::with_name("rom")
            .short("r")
            .long("rom")
            .value_name("ROM_PATH")
            .required(true)
            .help("Path to a Gameboy or Gameboy Color ROM")
            .takes_value(true))
        .arg(Arg::with_name("DISABLE_BOOT_ROM")
            .long("disable-boot-rom")
            .help("Disables the boot rom"))
        .arg(Arg::with_name("DEBUG")
            .long("debug")
            .help("Enables the debugger"))
        .get_matches();

    let rom = matches.value_of("rom").unwrap();
    if rom.len() < 1 {
        panic!("Must specify ROM to load");
    }

    let enable_debugger = matches.is_present("DEBUG");
    let disable_boot_rom = matches.is_present("DISABLE_BOOT_ROM");

    let cart = Cartridge::with_rom(load_rom(rom).unwrap());
    let mut gameboy = gameboy::GameBoy::new(false, cart, !disable_boot_rom);
    let game_title = gameboy.cart_details().game_title.clone();

    let mut window = create_window(game_title, enable_debugger);
    let mut ui = Ui::new(window.size().width as f64,
                         window.size().height as f64,
                         window.factory.clone());
    let debugger = Rc::new(RefCell::new(gameboy::debugger::Debugger::new()));
    let mut factory = window.factory.clone();

    'start: while let Some(e) = window.next() {
        if let Some(button) = e.press_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::Space => gameboy.set_speed(CpuSpeed::Double),
                    Key::A => gameboy.press(JoypadButton::A),
                    Key::S => gameboy.press(JoypadButton::B),
                    Key::Return => gameboy.press(JoypadButton::Start),
                    Key::Left => gameboy.press(JoypadButton::Left),
                    Key::Right => gameboy.press(JoypadButton::Right),
                    Key::Up => gameboy.press(JoypadButton::Up),
                    Key::Down => gameboy.press(JoypadButton::Down),
                    Key::Backspace => gameboy.reset(),
                    _ => (),
                }
            }
        }
        if let Some(button) = e.release_args() {
            if let Button::Keyboard(key) = button {
                match key {
                    Key::Space => gameboy.set_speed(CpuSpeed::Normal),
                    Key::A => gameboy.unpress(JoypadButton::A),
                    Key::S => gameboy.unpress(JoypadButton::B),
                    Key::Return => gameboy.unpress(JoypadButton::Start),
                    Key::Left => gameboy.unpress(JoypadButton::Left),
                    Key::Right => gameboy.unpress(JoypadButton::Right),
                    Key::Up => gameboy.unpress(JoypadButton::Up),
                    Key::Down => gameboy.unpress(JoypadButton::Down),
                    _ => (),
                }
            }
        }
        ui.handle_event(&e);
        window.draw_2d(&e, |mut c, g| {
            // clear([1.0; 4], g);
            let draw_state = c.draw_state.blend(Blend::Alpha);
            c.draw_state = draw_state;
            ui.draw(c, g);
            let img = {
                let frame = gameboy.request_frame();
                build_frame(frame)
            };
            let texture = Texture::from_image(&mut factory, &img, &TextureSettings::new()).unwrap();
            image(&texture, c.transform, g);
            if let Err(msg) = gameboy.run() {
                // Dump the last texture we had
                img.save("/Users/Simon/last_frame.png").unwrap();
                panic!(msg);
            }
            Some(())
        });
    }
}

fn load_rom(fname: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(fname)?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    Ok(contents)
}

fn create_window<S>(title: S, debugger_enabled: bool) -> PistonWindow
    where S: Into<String>
{
    let (width, height) = if debugger_enabled {
        (1280, 720)
    } else {
        (160, 144)
    };

    let mut window: PistonWindow = WindowSettings::new(format!("chemboy: {}", title.into()),
                                                       [width, height])
        .exit_on_esc(true)
        .opengl(OpenGL::V3_2)
        .build()
        .unwrap();

    window.set_max_fps(60);

    window
}

fn build_frame(frame: &Frame) -> RgbaImage {
    let mut img = ImageBuffer::new(gameboy::SCREEN_WIDTH as u32, gameboy::SCREEN_HEIGHT as u32);
    for x in 0..160 {
        for y in 0..144 {
            let frame_pixel = frame.pixels[160 * y + x];
            let p = image::Rgba([frame_pixel.r, frame_pixel.g, frame_pixel.b, 0xFF]);
            img.put_pixel(x as u32, y as u32, p);
        }
    }

    img
}