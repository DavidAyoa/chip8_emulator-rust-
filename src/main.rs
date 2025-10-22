use std::fs;
use std::env;
use minifb::{Key, Window, WindowOptions};
use rodio::{OutputStream, Sink, source::SineWave};
use std::time::Duration;

struct Chip8 {
    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    stack: Vec<u16>,
    delay_timer: u8,
    sound_timer: u8,
    display: [[bool; 64]; 32],
    keys: [bool; 16],
}

const FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80
];

impl Chip8 {
    fn new() -> Self {
        let mut memory = [0u8; 4096];
        let v = [0u8; 16];

        memory[0..80].copy_from_slice(&FONTSET);

        let stack = Vec::new();

        Self {
            memory,
            v,
            i: 0,
            pc: 0x200,
            stack,
            delay_timer: 0,
            sound_timer: 0,
            display: [[false; 64]; 32],
            keys: [false; 16]
        }
    }

    fn load_rom(&mut self, rom_path: &str) -> std::io::Result<()> {
        let rom_data = fs::read(rom_path)?;

        self.memory[0x200..0x200 + rom_data.len()].copy_from_slice(&rom_data);
        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        let high_byte = self.memory[self.pc as usize];
        let low_byte = self.memory[self.pc as usize + 1];

        let opcode = ((high_byte as u16) << 8) | (low_byte as u16);
        self.pc += 2;

        opcode
    }

    fn execute(&mut self, opcode: u16) {
        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => self.display = [[false; 64]; 32],
                    0x00EE => self.op_00ee(),
                    _ => println!("Unknown 0x0--- opcode: 0x{:04X}", opcode),
                }
            }

            0x1000 => {
                let nnn = opcode & 0x0FFF;
                self.pc = nnn;
            }

            0x6000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] =  nn;
            }

            0x7000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.v[x] = self.v[x].wrapping_add(nn);
            }

            0xA000 => {
                let nnn = opcode & 0x0FFF;
                self.i = nnn;
            }

            0xD000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let n = (opcode & 0x000F) as u8;
                self.op_dxyn(x, y, n);
            }

            0xB000 => {
                let nnn = opcode & 0x0FFF;
                self.op_bnnn(nnn);
            }
            
            0xC000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.op_cxnn(x, nn);
            }
            
            0xF000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = opcode & 0x00FF;
                
                match nn {
                    0x07 => self.op_fx07(x),
                    0x0A => self.op_fx0a(x),
                    0x15 => self.op_fx15(x),
                    0x18 => self.op_fx18(x),
                    0x1E => self.op_fx1e(x),
                    0x29 => self.op_fx29(x),
                    0x33 => self.op_fx33(x),
                    0x55 => self.op_fx55(x),
                    0x65 => self.op_fx65(x),
                    _ => println!("Unknown FX-- opcode: 0x{:04X}", opcode),
                }
            }

            // Conditional Skips...

            0x3000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.op_3xnn(x, nn);
            }
            
            0x4000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = (opcode & 0x00FF) as u8;
                self.op_4xnn(x, nn);
            }
            
            0x5000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                self.op_5xy0(x, y);
            }
            
            0x9000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                self.op_9xy0(x, y);
            }

            // Math Operations...

            0x8000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let y = ((opcode & 0x00F0) >> 4) as usize;
                let op = opcode & 0x000F;
                
                match op {
                    0x0 => self.op_8xy0(x, y),
                    0x1 => self.op_8xy1(x, y),
                    0x2 => self.op_8xy2(x, y),
                    0x3 => self.op_8xy3(x, y),
                    0x4 => self.op_8xy4(x, y),
                    0x5 => self.op_8xy5(x, y),
                    0x6 => self.op_8xy6(x, y),
                    0x7 => self.op_8xy7(x, y),
                    0xE => self.op_8xye(x, y),
                    _ => println!("Unknown 8XY_ opcode: 0x{:04X}", opcode),
                }
            }

            0x2000 => {
                let nnn = opcode & 0x0FFF;
                self.op_2nnn(nnn);
            }

            // Keyboard Handling...

            0xE000 => {
                let x = ((opcode & 0x0F00) >> 8) as usize;
                let nn = opcode & 0x00FF;
                
                match nn {
                    0x9E => self.op_ex9e(x),
                    0xA1 => self.op_exa1(x),
                    _ => println!("Unknown EX-- opcode: 0x{:04X}", opcode),
                }
            }

            _ => {
                println!("Unimplemented opcode: 0x{:04X}", opcode);
            }
        }
    }

    fn op_dxyn(&mut self, x: usize, y: usize, n: u8) {
        let x_start = (self.v[x] as usize) % 64;
        let y_start = (self.v[y] as usize) % 32;

        self.v[0xF] = 0;

        for row in 0..n {
            let sprite_byte = self.memory[(self.i + row as u16) as usize];

            for col in 0..8 {
                let bit = (sprite_byte >> (7 - col)) & 1;

                if bit == 1 {
                    let screen_x = (x_start + col) % 64;
                    let screen_y = (y_start + row as usize) % 32;

                    let old_pixel = self.display[screen_y][screen_x];
                    self.display[screen_y][screen_x] ^= true;

                    if old_pixel && !self.display[screen_y][screen_x] {
                        self.v[0xF] = 1;
                    }
                }
            }
        }
    }

    fn op_8xye(&mut self, x: usize, _y: usize) {
        self.v[0xF] = (self.v[x] >> 7) & 1;
        self.v[x] <<= 1;
    }

    fn op_8xy0(&mut self, x: usize, y: usize) {
        self.v[x] = self.v[y];
    }

    fn op_8xy1(&mut self, x: usize, y: usize) {
        self.v[x] |= self.v[y];
    }

    fn op_8xy2(&mut self, x: usize, y: usize) {
        self.v[x] &= self.v[y];
    }

    fn op_8xy3(&mut self, x: usize, y: usize) {
        self.v[x] ^= self.v[y];
    }

    fn op_8xy4(&mut self, x: usize, y: usize) {
        let sum = self.v[x] as u16 + self.v[y] as u16;
        self.v[0xF] = if sum > 0xFF { 1 } else { 0 };
        self.v[x] = sum as u8;
    }

    fn op_8xy5(&mut self, x: usize, y: usize) {
        self.v[0xF] = if self.v[x] >= self.v[y] { 1 } else { 0 };
        self.v[x] = self.v[x].wrapping_sub(self.v[y]);
    }    

    fn op_8xy6(&mut self, x: usize, _y: usize) {
        self.v[0xF] = self.v[x] & 1;
        self.v[x] >>= 1;
    }

    fn op_8xy7(&mut self, x: usize, y: usize) {
        self.v[0xF] = if self.v[y] >= self.v[x] { 1 } else { 0 };
        self.v[x] = self.v[y] - self.v[x];
    }

    fn op_3xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] == nn {
            self.pc += 2;
        }
    }

    fn op_4xnn(&mut self, x: usize, nn: u8) {
        if self.v[x] != nn {
            self.pc += 2;
        }
    }

    fn op_5xy0(&mut self, x: usize, y: usize) {
        if self.v[x] == self.v[y] {
            self.pc += 2;
        }
    }

    fn op_9xy0(&mut self, x: usize, y: usize) {
        if self.v[x] != self.v[y] {
            self.pc += 2;
        }
    }

    fn op_2nnn(&mut self, nnn: u16) {
        self.stack.push(self.pc);
        self.pc = nnn;
    }

    fn op_00ee(&mut self) {
        let popped_addr = self.stack.pop().expect("Couldn't pop addr from stack!");
        self.pc = popped_addr;
    }

    fn op_bnnn(&mut self, nnn: u16) {
        self.pc = nnn + (self.v[0x0] as u16)
    }

     fn op_cxnn(&mut self, x: usize, nn: u8) {
         let random_byte: u8 = rand::random();
         self.v[x] = random_byte & nn;
     }

     fn op_fx07(&mut self, x: usize) {
        self.v[x] = self.delay_timer;
    }

    fn op_fx15(&mut self, x: usize) {
        self.delay_timer = self.v[x];
    }

    fn op_fx18(&mut self, x: usize) {
        self.sound_timer = self.v[x];
    }

    fn op_fx1e(&mut self, x: usize) {
        self.i += self.v[x] as u16;
    }

    fn op_fx29(&mut self, x: usize) {
        self.i = (self.v[x] as u16) * 5;
    }

    fn op_fx33(&mut self, x: usize) {
        self.memory[self.i as usize] = self.v[x] / 100;
        self.memory[self.i as usize + 1] = (self.v[x] / 10) % 10;
        self.memory[self.i as usize + 2] = self.v[x] % 10;
    }

    fn op_fx55(&mut self, x: usize) {
        for misc in 0..=x {
            self.memory[self.i as usize + misc] = self.v[misc];
        }
    }

    fn op_fx65(&mut self, x: usize) {
        for misc in 0..=x {
            self.v[misc] = self.memory[self.i as usize + misc];
        }
    }

    fn op_ex9e(&mut self, x: usize) {
        if self.keys[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    fn op_exa1(&mut self, x: usize) {
        if !self.keys[self.v[x] as usize] {
            self.pc += 2;
        }
    }

    fn op_fx0a(&mut self, x: usize) {
        let mut key_pressed = false;
    
        for i in 0..16 {
            if self.keys[i] {
                self.v[x] = i as u8;
                key_pressed = true;
                break;
            }
        }
    
        if !key_pressed {
            self.pc = self.pc.saturating_sub(2);
        }
    }    

    fn get_display_buffer(&self) -> Vec<u32> {
        let mut buffer = vec![0u32; 64 * 32];
        
        for y in 0..32 {
            for x in 0..64 {
                let pixel = if self.display[y][x] {
                    0xFFFFFF
                } else {
                    0x000000
                };
                buffer[y * 64 + x] = pixel;
            }
        }
        
        buffer
    }

    fn update_keys(&mut self, window: &Window) {
        self.keys = [false; 16];
        
        self.keys[0x1] = window.is_key_down(Key::Key1);
        self.keys[0x2] = window.is_key_down(Key::Key2);
        self.keys[0x3] = window.is_key_down(Key::Key3);
        self.keys[0xC] = window.is_key_down(Key::Key4);
        
        self.keys[0x4] = window.is_key_down(Key::Q);
        self.keys[0x5] = window.is_key_down(Key::W);
        self.keys[0x6] = window.is_key_down(Key::E);
        self.keys[0xD] = window.is_key_down(Key::R);
        
        self.keys[0x7] = window.is_key_down(Key::A);
        self.keys[0x8] = window.is_key_down(Key::S);
        self.keys[0x9] = window.is_key_down(Key::D);
        self.keys[0xE] = window.is_key_down(Key::F);
        
        self.keys[0xA] = window.is_key_down(Key::Z);
        self.keys[0x0] = window.is_key_down(Key::X);
        self.keys[0xB] = window.is_key_down(Key::C);
        self.keys[0xF] = window.is_key_down(Key::V);
    }
}

fn main() {
    let mut chip8 = Chip8::new();
    
    // Get ROM from command line or use default
    let args: Vec<String> = env::args().collect();
    let rom_path = if args.len() > 1 { &args[1] } else { "Pong.ch8" };
    
    println!("╔═════════════════════════════════════════════╗");
    println!("║   CHIP-8 EMULATOR - RUST EDITION BY INCENIX ║");
    println!("╚═════════════════════════════════════════════╝");
    println!("\nLoading ROM: {}", rom_path);
    
    match chip8.load_rom(rom_path) {
        Ok(_) => println!("✓ ROM loaded successfully!\n"),
        Err(e) => {
            eprintln!("✗ Failed to load ROM: {}", e);
            eprintln!("\nUsage: cargo run [rom_path]");
            return;
        }
    }

    let mut window = Window::new(
        "Chip-8 Emulator",
        64 * 10,
        32 * 10,
        WindowOptions::default(),
    )
    .expect("Failed to create window");

    window.limit_update_rate(Some(Duration::from_micros(16600)));

    // Audio setup
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();
    let mut beeping = false;
    
    let mut instructions_per_frame = 10;

    println!("Controls:");
    println!("  ESC      - Exit emulator");
    println!("  +/=      - Speed up");
    println!("  -        - Slow down");
    println!("  1234     - Keys 1, 2, 3, C");
    println!("  QWER     - Keys 4, 5, 6, D");
    println!("  ASDF     - Keys 7, 8, 9, E");
    println!("  ZXCV     - Keys A, 0, B, F");
    println!("\nEmulator running...\n");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        chip8.update_keys(&window);
        
        if window.is_key_pressed(Key::Equal, minifb::KeyRepeat::No) {
            instructions_per_frame = (instructions_per_frame + 2).min(50);
            println!("Speed: {}x", instructions_per_frame / 10);
        }
        if window.is_key_pressed(Key::Minus, minifb::KeyRepeat::No) {
            instructions_per_frame = (instructions_per_frame - 2).max(2);
            println!("Speed: {}x", instructions_per_frame / 10);
        }
        
        for _ in 0..instructions_per_frame {
            if chip8.pc >= 4094 {
                break;
            }
            
            let opcode = chip8.fetch();
            chip8.execute(opcode);
        }

        if chip8.delay_timer > 0 {
            chip8.delay_timer -= 1;
        }
        if chip8.sound_timer > 0 {
            chip8.sound_timer -= 1;
            
            if !beeping {
                sink.append(SineWave::new(440.0));
                beeping = true;
            }
        } else if beeping {
            sink.stop();
            beeping = false;
        }

        let buffer = chip8.get_display_buffer();
        window
            .update_with_buffer(&buffer, 64, 32)
            .expect("Failed to update window");
    }
    
    println!("\nEmulator closed. Thanks for playing!");
}