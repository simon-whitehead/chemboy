extern crate clap;

use clap::{App, Arg, SubCommand};

mod gameboy;

fn main() {
    let app = App::new("gameboy-rs")
        .version("0.0.0.1")
        .author("Simon Whitehead")
        .about("A GameBoy and GameBoy Colour emulator written in Rust")
        .get_matches();

    let gameboy = gameboy::GameBoy::new();
}
