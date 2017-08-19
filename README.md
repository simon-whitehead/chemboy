### chemboy
-------------

A GameBoy (and hopefully in future, GameBoy Colour) emulator written in Rust.

**This is very much a work in progress** and can only render non-Gameboy Colour games. It can only render Tetris up to the main menu and slightly beyond before crashing. If you're looking for a proper working emulator, you've come to the wrong place :)

### Progress GIF

Here is the latest progress GIF. We have Tetrominos! We are able to play a game of Tetris until we want to start a new game.

The score seems to be a little bit bugged too the longer a game goes on - so that needs investigating.

![chemboy-progress-19-08-2017](https://user-images.githubusercontent.com/2499070/29483787-32c5c180-84f3-11e7-9305-13dc4827759f.gif)

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