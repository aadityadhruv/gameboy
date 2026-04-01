use clap::Parser;
mod emulator;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Name of the ROM file
    #[arg(name = "ROM")]
    name: String,
}


fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem.window("Cartridge", 160, 144)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let args = Args::parse();
    let mut emulator = emulator::emulator::Emulator::new();
    emulator.mmu.load_rom(&args.name);
    loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return;
                },
                _ => {}
            }
        }
        emulator.cpu.fetch(&mut emulator.mmu);
        emulator.cpu.execute(&mut emulator.mmu);
        canvas.present();
    }
}
