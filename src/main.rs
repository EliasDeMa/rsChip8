extern crate sdl2;
extern crate rand;

pub mod cpu;
pub mod font;
pub mod displaydriver;
pub mod inputdriver;

use cpu::Cpu;
use std::fs::File;
use std::io::Read;
use displaydriver::DisplayDriver;
use inputdriver::InputDriver;
use std::thread;
use std::time::Duration;

pub const CHIP8_WIDTH: usize = 64;
pub const CHIP8_HEIGHT: usize =  32;
pub const CHIP8_RAM: usize = 4096;

fn main() {
    let sleep_duration = Duration::from_millis(2);
    let mut f = File::open("data/PONG2").unwrap();
    let mut data = Vec::<u8>::new();
    f.read_to_end(&mut data).expect("Unable to read file");

    let sdl_context = sdl2::init().unwrap();

    let mut chip8 = Cpu::new();
    let mut display_driver = DisplayDriver::new(&sdl_context);
    let mut input_driver = InputDriver::new(&sdl_context);

    chip8.load(&data); 
    while let Ok(keypad) = input_driver.poll() {

        let output = chip8.tick(keypad);

        if output.vram_changed {
            display_driver.draw(output.vram);
        }

        thread::sleep(sleep_duration);
    }
}