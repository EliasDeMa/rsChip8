use font::FONT_SET;

use HEIGHT;
use RAM;
use WIDTH;
use rand::prelude::*;

pub struct OutputState<'a> {
    pub vram: &'a [[u8; WIDTH]; HEIGHT],
    pub vram_changed: bool,
}

pub struct Cpu {
    vram: [[u8; WIDTH]; HEIGHT],
    vram_changed: bool,
    ram: [u8; RAM],
    stack: [usize; 16],
    v: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
    keypad_waiting: bool,
    keypad_register: usize,
}

impl Cpu {
    pub fn new() -> Self {

        let mut ram = [0u8; RAM];
        for i in 0..FONT_SET.len() {
            ram[i] = FONT_SET[i];
        }

        Cpu {
            vram: [[0; WIDTH]; HEIGHT],
            vram_changed: false,
            ram: ram,
            stack: [0; 16],
            v: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
        }
    }

    pub fn load(&mut self, data: &[u8]) {
        for (i, &byte) in data.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < 4096 {
                self.ram[0x200 + i] = byte;
            } else {
                break;
            }
        }
    }

    pub fn tick(&mut self, keypad: [bool; 16]) -> OutputState {
        self.keypad = keypad;
        self.vram_changed = false;

        if self.keypad_waiting {
            for i in 0..keypad.len() {
                if keypad[i] {
                    self.keypad_waiting = false;
                    self.v[self.keypad_register] = i as u8;
                    break;
                }
            }
        } else {
            if self.delay_timer > 0 {
                self.delay_timer -= 1
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1
            }
            let opcode = self.get_opcode();
            self.run_opcode(opcode);
        }

        OutputState {
            vram: &self.vram,
            vram_changed: self.vram_changed,
        }
    }

    pub fn get_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    pub fn run_opcode(&mut self, opcode: u16) {
        
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        match nibbles.0 {
            0x0 => {
                self.match0x0(kk);
            },
            0x1 => {
                self.pc = nnn;
            },
            0x2 => {
                self.stack[self.sp] = self.pc + 2;
                self.sp += 1;
                self.pc = nnn;
            },
            0x3 => {
                if self.v[x] == kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0x4 => {
                if self.v[x] != kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            0x6 => {
                self.v[x] = kk;
                self.pc += 2;
            },
            0x7 => {
                let vx = self.v[x] as u16;
                let val = kk as u16;
                let result = vx + val;
                self.v[x] = result as u8;
                self.pc += 2;
            },
            0x8 => {
                self.match0x8(n, x, y);
            },
            0x9 => {
                if self.v[x] != self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            },
            0xA => {
                self.i = nnn;
                self.pc += 2;
            },
            0xB => {
                self.pc = (self.v[0] as usize) + nnn;
            },
            0xC => {
                let mut rng = thread_rng();
                self.v[x] = rng.gen::<u8>() & kk;
                self.pc += 2;
            },
            0xD => {
                self.v[0x0f] = 0;
                for byte in 0..n {
                    let y = (self.v[y] as usize + byte) % HEIGHT;
                    for bit in 0..8 {
                        let x = (self.v[x] as usize + bit) % WIDTH;
                        let color = (self.ram[self.i + byte] >> (7 - bit)) & 1;
                        self.v[0x0f] |= color & self.vram[y][x];
                        self.vram[y][x] ^= color;
                    }
                }
                self.vram_changed = true;
                self.pc += 2;
            },
            0xE => {
                self.match0xe(kk, x);
            },
            0xF => {
                self.match0xf(kk, x);
            },
            _ => panic!("Uninplemented instruction {:#X} ", nibbles.0),
        }
    }

    fn match0x0(&mut self, kk: u8) {
        match kk {
            0xE0 => {
                for y in 0..HEIGHT {
                    for x in 0..WIDTH {
                        self.vram[y][x] = 0;
                        }
                    }
                self.vram_changed = true;
                self.pc += 2;
            },
            0xEE => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            },
            _ => panic!("uninplemented 00** {:#X} ", kk),
        }
    }

    fn match0x8(&mut self, n: usize, x: usize, y: usize) {
        match n {
            0x0 => {
                let vy = self.v[y];
                self.v[x] = vy;                    
            },
            0x1 => {
                self.v[x] |= self.v[y];        
                    },
            0x2 => {   
                self.v[x] &= self.v[y];                        
                    },
            0x3 => {
                self.v[x] ^= self.v[y];         
                    },
            0x4 => {
                let vx = self.v[x] as u16;
                let vy = self.v[y] as u16;
                let result = vx + vy;
                self.v[x] = result as u8;
                self.v[0x0f] = if result > 0xFF { 1 } else { 0 };    
                    },
            0x5 => {
                self.v[0x0f] = if self.v[x] > self.v[y] { 1 } else { 0 };
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);         
                    },
            0x6 => {
                self.v[0x0f] = self.v[x] & 1;
                self.v[x] >>= 1;      
                    },
            0x7 => {
                self.v[0x0f] = if self.v[y] > self.v[x] { 1 } else { 0 };
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);         
                    },
            0xE => {
                self.v[0x0f] = (self.v[x] & 0b10000000) >> 7;
                self.v[x] <<= 1;
            },
            _ => panic!("uninplemented 8XY* {} ", n),
        };
        self.pc += 2;
    }

    fn match0xe(&mut self, kk: u8, x: usize) {
        match kk {
            0x9E => {
                if self.keypad[self.v[x] as usize] {
                    self.pc += 2;
                } 
            },
            0xA1 => {
                if !self.keypad[self.v[x] as usize] {
                    self.pc += 2;
                } 
            },
            _ => panic!("uninplemented isntruction EX** {:#X}", kk),
        }
        self.pc += 2;
    }

    fn match0xf(&mut self, kk: u8, x: usize) {
        match kk {
            0x7 => {
                self.v[x] = self.delay_timer;
            },
            0xA => {
                self.keypad_waiting = true;
                self.keypad_register = x;
            },
            0x15 => {
                self.delay_timer = self.v[x];
            },
            0x18 => {
                self.sound_timer = self.v[x];
            },
            0x1E => {
                let vx = self.v[x];
                self.i += vx as usize;
            },
            0x29 => {
                self.i = (self.v[x] as usize) * 5;
            },
            0x33 => {
                self.ram[self.i] = self.v[x] / 100;
                self.ram[self.i + 1] = (self.v[x] % 100) / 10;
                self.ram[self.i + 2] = self.v[x] % 10;
            },
            0x55 => {
                for i in 0..x + 1 {
                    self.ram[self.i + i] = self.v[i];
                }
            },
            0x65 => {
                for i in 0..x+1 {
                    self.v[i] = self.ram[self.i + 1];
                }

            },
            _ => panic!("Uninplemented instruction FX** {:#X} ", kk),
        }
        self.pc += 2;
    }
}

