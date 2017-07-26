extern crate byteorder;
extern crate clap;

use clap::{App, Arg, SubCommand};

use std::fs::File;
use std::io::Read;

pub mod gameboy;

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
        .get_matches();

    let rom = matches.value_of("rom").unwrap();
    let rom_data = load_rom(rom).unwrap();
    let mut gameboy = gameboy::GameBoy::new(false, rom_data);

    'init: loop {
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
