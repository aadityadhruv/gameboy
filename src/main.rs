use clap::Parser;
mod cpu;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Name of the ROM file
    #[arg(short, long)]
    name: String,
}


fn main() {
    let args = Args::parse();
    let mut cpu = cpu::chip::Chip::new();
    cpu.load_rom(&args.name);
    loop {
        cpu.fetch();
        cpu.execute();
    }
}
