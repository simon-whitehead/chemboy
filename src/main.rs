extern crate byteorder;
extern crate clap;
#[macro_use]
extern crate conrod;
extern crate gfx_core;
extern crate gfx_device_gl;
extern crate image;
extern crate piston_window;
extern crate rand;

use conrod::{Colorable, Labelable, Positionable, Sizeable, Widget};
use clap::{App, Arg};
use image::{ImageBuffer, RgbaImage};
use gfx_device_gl::Factory;
use piston_window::*;

use piston_window::{PistonWindow, UpdateEvent, Window, WindowSettings};
use piston_window::{Flip, G2d, G2dTexture, Texture, TextureSettings};
use piston_window::OpenGL;
use piston_window::texture::UpdateTexture;


use std::cell::Cell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;
use std::thread;
use std::time::Instant;

pub mod gameboy;

use gameboy::{Cartridge, CpuSpeed, Frame, JoypadButton};

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
    let debugger = if enable_debugger {
        Some(Rc::new(gameboy::debugger::Debugger::new()))
    } else {
        None
    };

    let disable_boot_rom = matches.is_present("DISABLE_BOOT_ROM");
    let rom_data = load_rom(rom).unwrap();
    let cart = Cartridge::with_rom(rom_data);
    let mut gameboy = gameboy::GameBoy::new(false, cart, !disable_boot_rom);
    println!("Loading game: {}", gameboy.cart_details().game_title);
    let now = Instant::now();

    let opengl = OpenGL::V3_2;
    let (width, height) = if enable_debugger {
        (1280, 720)
    } else {
        (160, 144)
    };// Generate a unique `WidgetId` for each widget.
    widget_ids! {
    pub struct Ids {
        canvas,
        test_button,
        introduction
    }
}

    let screen_size = Rc::new(Cell::new(0.5));
    let mut window: PistonWindow = WindowSettings::new(format!("chemboy: {}",
                                                               gameboy.cart_details().game_title),
                                                       [width, height])
        .exit_on_esc(true)
        .opengl(opengl)
        .build()
        .unwrap();

    window.set_max_fps(60);
    let mut ui = conrod::UiBuilder::new([1280.0, 720.0]).theme(theme()).build();
    let mut factory = window.factory.clone();
    // Create a texture to use for efficiently caching text on the GPU.
    let mut text_vertex_data = Vec::new();
    let (mut glyph_cache, mut text_texture_cache) = {
        const SCALE_TOLERANCE: f32 = 0.1;
        const POSITION_TOLERANCE: f32 = 0.1;
        let cache = conrod::text::GlyphCache::new(1280, 720, SCALE_TOLERANCE, POSITION_TOLERANCE);
        let buffer_len = 1280 as usize * 720 as usize;
        let init = vec![128; buffer_len];
        let settings = TextureSettings::new();
        let factory = &mut window.factory;
        let texture = G2dTexture::from_memory_alpha(factory, &init, 1280, 720, &settings).unwrap();
        (cache, texture)
    };
    let mut image_map = conrod::image::Map::new();

    let ids = Ids::new(ui.widget_id_generator());
    let mut bg_color = conrod::color::LIGHT_BLUE;
    let n = 0;
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
        // Convert the piston event to a conrod event.
        let size = window.size();
        let (win_w, win_h) = (size.width as conrod::Scalar, size.height as conrod::Scalar);
        if let Some(evt) = conrod::backend::piston::event::convert(e.clone(), win_w, win_h) {
            ui.handle_event(evt);
        }

        let screen_size_clone = screen_size.clone();

        e.update(|_| {
            let mut ui = ui.set_widgets();
            conrod::widget::Canvas::new()
                .pad(30.0)
                .color(bg_color)
                .w_h(500.0, 500.0)
                .set(ids.canvas, &mut ui);
            const INTRODUCTION: &'static str =
                "This example aims to demonstrate all widgets that are provided by conrod.\n\nThe \
                 widget that you are currently looking at is the Text widget. The Text widget is \
                 one of several special \"primitive\" widget types which are used to construct \
                 all other widget types. These types are \"special\" in the sense that conrod \
                 knows how to render them via `conrod::render::Primitive`s.\n\nScroll down to see \
                 more widgets!";
            conrod::widget::Text::new(INTRODUCTION)
                .padded_w_of(ids.canvas, 20.0)
                .down(60.0)
                .align_middle_x_of(ids.canvas)
                .center_justify()
                .line_spacing(5.0)
                .set(ids.introduction, &mut ui);
            if conrod::widget::Button::new()
                .label("Click me")
                .middle_of(ids.canvas)
                .w_h(130.0, 130.0)
                .set(ids.test_button, &mut ui)
                .was_clicked() {
                screen_size_clone.set(screen_size_clone.get() * 2.0);
            }
        });
        window.draw_2d(&e, |c, g| {
            // clear([1.0; 4], g);
            if let Some(primitives) = ui.draw_if_changed() {
                // A function used for caching glyphs to the texture cache.
                let cache_queued_glyphs = |graphics: &mut G2d,
                                           cache: &mut G2dTexture,
                                           rect: conrod::text::rt::Rect<u32>,
                                           data: &[u8]| {
                    let offset = [rect.min.x, rect.min.y];
                    let size = [rect.width(), rect.height()];
                    let format = piston_window::texture::Format::Rgba8;
                    let encoder = &mut graphics.encoder;
                    text_vertex_data.clear();
                    text_vertex_data.extend(data.iter().flat_map(|&b| vec![255, 255, 255, b]));
                    UpdateTexture::update(cache,
                                          encoder,
                                          format,
                                          &text_vertex_data[..],
                                          offset,
                                          size)
                        .expect("failed to update texture")
                };
                // Specify how to get the drawable texture from the image. In this case, the image
                // *is* the texture.
                fn texture_from_image<T>(img: &T) -> &T {
                    img
                }

                // Draw the conrod `render::Primitives`.
                conrod::backend::piston::draw::primitives(primitives,
                                                          c,
                                                          g,
                                                          &mut text_texture_cache,
                                                          &mut glyph_cache,
                                                          &image_map,
                                                          cache_queued_glyphs,
                                                          texture_from_image);
            }

            let img = {
                let frame = gameboy.request_frame();
                build_frame(frame)
            };
            let texture = Texture::from_image(&mut factory, &img, &TextureSettings::new()).unwrap();
            image(&texture,
                  c.transform.scale(screen_size.get(), screen_size.get()),
                  g);
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

/// A set of reasonable stylistic defaults that works for the `gui` below.
pub fn theme() -> conrod::Theme {
    use conrod::position::{Align, Direction, Padding, Position, Relative};
    conrod::Theme {
        name: "Demo Theme".to_string(),
        padding: Padding::none(),
        x_position: Position::Relative(Relative::Align(Align::Start), None),
        y_position: Position::Relative(Relative::Direction(Direction::Backwards, 20.0), None),
        background_color: conrod::color::GREEN,
        shape_color: conrod::color::LIGHT_CHARCOAL,
        border_color: conrod::color::BLACK,
        border_width: 0.0,
        label_color: conrod::color::WHITE,
        font_id: None,
        font_size_large: 26,
        font_size_medium: 18,
        font_size_small: 12,
        widget_styling: conrod::theme::StyleMap::default(),
        mouse_drag_threshold: 0.0,
        double_click_threshold: std::time::Duration::from_millis(500),
    }
}