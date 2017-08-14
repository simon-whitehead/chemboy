### chemboy
-------------

A GameBoy (and hopefully in future, GameBoy Colour) emulator written in Rust.

This is very much a work in progress and can only render non-Gameboy Colour games. It can only render Tetris up to the main menu and slightly beyond before crashing. If you're looking for a proper working emulator, you've come to the wrong place :)

### Progress GIF

Here is the latest progress GIF. Currently, we can render Tetris and make it to the point where we actually want to start a game of Tetris.

![Progress GIF](https://user-images.githubusercontent.com/2499070/29275851-e3439688-814f-11e7-8120-37298e224e7b.gif)

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