### chemboy
-------------

A GameBoy (and hopefully in future, GameBoy Colour) emulator written in Rust.

**This is very much a work in progress** and can only render non-Gameboy Colour games. It can only render Tetris up to the main menu and slightly beyond before crashing. If you're looking for a proper working emulator, you've come to the wrong place :)

### Progress GIF

Here is the latest progress GIF. We can play Tetris!

I have implemented 243 base instructions and all 256 of the extended instruction range. This means we're only a couple of instructions away from full support of the Gameboy CPU. As such, we can now play Tetris without it crashing at all. The progress GIF below shows a 2 game session of Tetris.

The score is still a little bit buggy. Not sure whats going on there, but thats the next thing to figure out.

![chemboy-progress-22-08-2017](https://user-images.githubusercontent.com/2499070/29525895-74f411e8-86d7-11e7-9c31-1ef7d3c27c24.gif)

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