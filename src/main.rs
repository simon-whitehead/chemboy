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

use image::Pixel;
use clap::{App, Arg};
use image::ImageBuffer;
use piston_window::*;
use piston_window::OpenGL;

use std::fs::File;
use std::io::Read;
use std::ops::{Deref, DerefMut};

pub mod gameboy;

use gameboy::{Cartridge, CpuSpeed, Frame, JoypadButton, Ui};

const WINDOW_WIDTH: u32 = 180;
const WINDOW_HEIGHT: u32 = 180;

fn main() {
    let matches = App::new("chemboy")
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
        .arg(
            Arg::with_name("DISABLE_BOOT_ROM")
                .long("disable-boot-rom")
                .help("Disables the boot rom"),
        )
        .arg(
            Arg::with_name("DEBUG")
                .long("debug")
                .help("Enables the debugger"),
        )
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

    let mut window = create_window(game_title, enable_debugger);
    let mut factory = window.factory.clone();

    /*let mut ui = Ui::new(
        window.size().width as f64,
        window.size().height as f64,
        window.factory.clone(),
        &rom,
    );
    let debugger = Rc::new(RefCell::new(gameboy::debugger::Debugger::new()));*/

    let mut imgbuf = ImageBuffer::new(gameboy::SCREEN_WIDTH as u32, gameboy::SCREEN_HEIGHT as u32);
    build_frame(&mut imgbuf, gameboy.request_frame());
    let mut texture = Texture::from_image(&mut factory, &imgbuf, &TextureSettings::new())
        .expect("err: could not build requested gameboy frame");

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
        /*let ui_event = ui.handle_event(&e);
        match ui_event {
            UIEvent::ThemeSwitched(theme) => gameboy.switch_theme(theme),
            _ => (),
        }*/
        texture.update(&mut window.encoder, &imgbuf);
        window.draw_2d(&e, |c, g| {
            //ui.draw(c, g);
            build_frame(&mut imgbuf, gameboy.request_frame());
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

fn create_window<S>(title: S, debugger_enabled: bool) -> PistonWindow
where
    S: Into<String>,
{
    let mut window: PistonWindow = WindowSettings::new(
        format!("chemboy: {}", title.into()),
        [WINDOW_WIDTH, WINDOW_HEIGHT],
    ).exit_on_esc(true)
        .opengl(OpenGL::V3_2)
        .build()
        .unwrap();

    window.set_max_fps(60);

    window
}

fn build_frame<P, C>(imgbuf: &mut ImageBuffer<P, C>, frame: &Frame)
where
    P: Pixel + From<image::Rgba<u8>> + 'static,
    P::Subpixel: 'static,
    C: Deref<Target = [P::Subpixel]> + DerefMut,
{
    for x in 0..gameboy::SCREEN_WIDTH {
        for y in 0..gameboy::SCREEN_HEIGHT {
            let frame_pixel = frame.pixels[gameboy::SCREEN_WIDTH * y + x];
            let p: P = image::Rgba([frame_pixel.r, frame_pixel.g, frame_pixel.b, 0xFF]).into();
            imgbuf.put_pixel(x as u32, y as u32, p);
        }
    }
}
