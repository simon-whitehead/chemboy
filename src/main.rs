extern crate byteorder;
extern crate clap;

use clap::{App, Arg, SubCommand};

use std::fs::File;
use std::io::Read;
use std::time::{Duration, Instant};

pub mod gameboy;

use gameboy::cartridge::Cartridge;

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
    let cart = Cartridge::with_rom(rom_data);
    let mut gameboy = gameboy::GameBoy::new(false, cart);
    println!("Loading game: {}", gameboy.cart_details().game_title);
    let mut now = Instant::now();

    'init: loop {
        if Instant::now().duration_since(now) > Duration::new(1, 0) {
            println!("1 second has elapsed (STUB HERE FOR V-BLANK DEBUGGING)");
            now = Instant::now();
        }
        match gameboy.run() {
            false => break 'init,
            _ => (),
        }
    }
}

fn load_rom(fname: &str) -> std::io::Result<Vec<u8>> {
    let mut file = File::open(fname)?;

    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    Ok(contents)
}
