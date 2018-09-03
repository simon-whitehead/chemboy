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
use piston_window::*;
use piston_window::OpenGL;

use std::cell::{Cell, RefCell};
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::thread;

pub mod gameboy;

use gameboy::{Cartridge, CpuSpeed, Frame, JoypadButton, Ui};
use gameboy::ui::ui_event::UIEvent;
use gameboy::ui::theme::Theme;

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

fn main() {
    let matches = App::new("chemboy")
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

    let rom = load_rom(rom).unwrap();
    let cart = Cartridge::with_rom(&rom);
    let mut gameboy = gameboy::GameBoy::new(false, cart, !disable_boot_rom);
    let game_title = gameboy.cart_details().game_title.clone();

    let mut game_window = create_window(game_title, (WINDOW_WIDTH, WINDOW_HEIGHT));

    let debugger = Rc::new(RefCell::new(gameboy::debugger::Debugger::new()));
    let mut factory = game_window.factory.clone();

    let mut ui = Ui::new(600.0, 400.0, game_window.factory.clone(), &rom);
    let mut debugger_closed = false;

    let mut debugger_window = create_window("debugger", (600, 400));
    debugger_window.events = game_window.events.clone();

    'start: while let Some(e) = game_window.next() {
        for evt in debugger_window.next() {
            debugger_window.draw_2d(&evt, |c, g| {
                clear([1.0, 1.0, 0.0, 1.0], g);
                // ui.draw(c, g);
                Some(())
            });

        }
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
            println!("keypress");
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
        game_window.draw_2d(&e, |c, g| {
            let frame = build_frame(gameboy.request_frame());
            let texture = Texture::from_image(&mut factory, &frame, &TextureSettings::new())
                .expect("err: could not build requested gameboy frame");
            let (x, y) = get_projection_coordinates();
            image(&texture, c.transform.trans(x, y), g);
            gameboy.run();
            Some(())
        });
    }
}

fn get_projection_coordinates() -> (f64, f64) {
    let x = (WINDOW_WIDTH as f64 / 2.0) - (gameboy::SCREEN_WIDTH as f64 / 2.0);
    let y = 25.0;

    (x, y)
}

fn load_rom(fname: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(fname)?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    Ok(contents)
}

fn create_window<S>(title: S, dimensions: (u32, u32)) -> PistonWindow
    where S: Into<String>
{
    let mut window: PistonWindow = WindowSettings::new(format!("chemboy: {}", title.into()),
                                                       [dimensions.0, dimensions.1])
        .exit_on_esc(true)
        .opengl(OpenGL::V3_2)
        .build()
        .unwrap();

    window.set_max_fps(60);

    window
}

fn build_frame(frame: &Frame) -> RgbaImage {
    let mut img = ImageBuffer::new(gameboy::SCREEN_WIDTH as u32, gameboy::SCREEN_HEIGHT as u32);
    for x in 0..gameboy::SCREEN_WIDTH {
        for y in 0..gameboy::SCREEN_HEIGHT {
            let frame_pixel = frame.pixels[gameboy::SCREEN_WIDTH * y + x];
            let p = image::Rgba([frame_pixel.r, frame_pixel.g, frame_pixel.b, 0xFF]);
            img.put_pixel(x as u32, y as u32, p);
        }
    }

    img
}