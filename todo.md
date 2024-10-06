### Emulator features
- Tracelogger
- Support for cheats
- Rewind/save state functionality
- Memory editor
- Sprite/background viewer
- Support for shaders to change the video output
- Scripting support 
- TAS replay support


### Assembler & Engine Ideas

Rust based assembler
Rust/C/C++ Engine 

#### Structure

src
|
main.rs - inits
chip/|
	 |
	 |
	 gameboy.rs - loop, Opcode, buffers etc.
	 dassm.rs   - wrapper struct for gameboy
graphics/|
		 |
		 |
		 graphics.rs - generic lib which is called in main.rs
		 sdl2.rs - Using sdl2 as the backend
		 optional.rs - some other backend
input/|
	  |
	  |
	  input.rs generic lib called in main.rs
	  sdl2.rs - sdl2 input


### Implementation Order

1. CPU
2. Input
3. Interrupts
4. Graphics
5. Audio
6. Peripherals
7. Engine
