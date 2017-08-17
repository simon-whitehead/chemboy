### chemboy
-------------

A GameBoy (and hopefully in future, GameBoy Colour) emulator written in Rust.

**This is very much a work in progress** and can only render non-Gameboy Colour games. It can only render Tetris up to the main menu and slightly beyond before crashing. If you're looking for a proper working emulator, you've come to the wrong place :)

### Progress GIF

Here is the latest progress GIF. We have Tetrominos! Unfortunately the emulator crashes once the Tetromino hits the bottom due to an unknown opcode. Also, it _always_ renders a square Tetromino for the moment. Tetris uses the `TIMA` register as a seed for randomly selecting a Tetromino - so proper testing of that is required.

![chemboy-progress-17-08-2017](https://user-images.githubusercontent.com/2499070/29411349-d27b52c4-8397-11e7-917b-571ae1925bea.gif)

### Project goals

First and foremost this is a learning project for myself.

Ideally, this emulator will be able to run Tetris in both the DMG and GBC versions. After that, I will move on to other games.

### Compiling

If you want decent performance, you should compile and run this emulator with the `--release` flag:

```
cargo run --release -- --rom /path/to/rom.gb
```

### Contributing

If you want to contribute, please open an issue and discuss your planned contribution. I don't actually have a solid roadmap other than "support DMG Tetris" at the moment, so discussion around features is a good idea.