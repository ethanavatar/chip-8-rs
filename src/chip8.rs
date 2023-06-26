use rand::Rng;
use sdl2::keyboard::Keycode;

const SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Chip8 {
    pub screen: [u8; 64 * 32],
    keys: [u8; 16],

    memory: [u8; 4096],
    v: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    delay_timer: u8,
    sound_timer: u8,

    paused: bool,
    clock_speed: u32,
    waiting_for_key: bool
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            screen: [0; 64 * 32],
            keys: [0; 16],

            memory: [0; 4096],
            v: [0; 16],
            i: 0,
            pc: 0,
            sp: 0,
            stack: [0; 16],
            delay_timer: 0,
            sound_timer: 0,

            paused: false,
            clock_speed: 10,
            waiting_for_key: false
        }
    }

    pub fn reset(&mut self) {
        self.screen = [0; 64 * 32];
        self.keys = [0; 16];

        self.memory = [0; 4096];
        self.v = [0; 16];
        self.i = 0;
        self.pc = 0x200;
        self.sp = 0;
        self.stack = [0; 16];
        self.delay_timer = 0;
        self.sound_timer = 0;

        self.paused = false;
        self.clock_speed = 10;
        self.waiting_for_key = false;

        for i in 0..SPRITES.len() {
            self.memory[i] = SPRITES[i];
        }
    }

    fn set_pixel(&mut self, x: u8, y: u8) -> bool {
        self.screen[(x as usize) + (y as usize) * 64] ^= 1;
        self.screen[(x as usize) + (y as usize) * 64] == 0
    }

    fn clear_screen(&mut self) {
        self.screen = [0; 64 * 32];
    }

    pub fn set_key(&mut self, keycode: Keycode, value: u8) {
        let mut key = match keycode {
            Keycode::Num1 => Some(0x1),
            Keycode::Num2 => Some(0x2),
            Keycode::Num3 => Some(0x3),
            Keycode::Num4 => Some(0xc),
            Keycode::Q => Some(0x4),
            Keycode::W => Some(0x5),
            Keycode::E => Some(0x6),
            Keycode::R => Some(0xd),
            Keycode::A => Some(0x7),
            Keycode::S => Some(0x8),
            Keycode::D => Some(0x9),
            Keycode::F => Some(0xe),
            Keycode::Z => Some(0xa),
            Keycode::X => Some(0x0),
            Keycode::C => Some(0xb),
            Keycode::V => Some(0xf),
            _ => None
        };

        if let Some(keycode) = key {
            self.keys[keycode as usize] = value;

            if self.waiting_for_key && value == 1 {
                let opcode = (self.memory[self.pc as usize] as u16) << 8
                                | (self.memory[self.pc as usize + 1] as u16);
                self.v[((opcode & 0x0F00) >> 8) as usize] = keycode;
                self.paused = false;
            }
        }
    }

    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.reset();
        for i in 0..rom.len() {
            self.memory[0x200 + i] = rom[i];
        }
    }

    pub fn clock(&mut self) {
        if self.paused {
            return;
        }

        for _ in 0..self.clock_speed {
            let opcode = (self.memory[self.pc as usize] as u16) << 8
                            | (self.memory[self.pc as usize + 1] as u16);
            self.execute(opcode);
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            //println!("BEEP!");
        }
    }

    fn stack_push(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    fn execute(&mut self, opcode: u16) {

        self.pc += 2;

        let x = (opcode & 0x0F00) >> 8;
        let y = (opcode & 0x00F0) >> 4;

        let kk = (opcode & 0xFF) as u8;

        match opcode & 0xF000 {
            0x0000 => {
                match opcode {
                    0x00E0 => {
                        // Clear screen
                        //println!("{:X}\t|\tCLS", self.pc);
                        self.clear_screen()
                    },
                    0x00EE => {
                        // Return from subroutine
                        //println!("{:X}\t|\tRET", self.pc);
                        self.pc = self.stack_pop();
                    },
                    _ => println!("{:X}\t|\tSYS {:03X}", self.pc, opcode & 0xFFF)
                }
            },
            0x1000 => {
                // 0x1NNN: Jump to address NNN
                //println!("{:X}\t|\tJP {:03X}", self.pc, opcode & 0xFFF);
                self.pc = opcode & 0xFFF;
            },
            0x2000 => {
                // 0x2NNN: Call subroutine at NNN
                //println!("{:X}\t|\tCALL {:03X}", self.pc, opcode & 0xFFF);
                self.stack_push(self.pc);
                self.pc = opcode & 0xFFF;
            },
            0x3000 => {
                // 0x3XKK: Skip next instruction if VX == KK
                //println!("{:X}\t|\tSE V[{:X}], {:02X}", self.pc, x, kk);
                if self.v[x as usize] == kk {
                    self.pc += 2;
                }
            },
            0x4000 => {
                // 0x3XKK: Skip next instruction if VX != KK
                //println!("{:X}\t|\tSNE V[{:X}], {:02X}", self.pc, x, kk);
                if self.v[x as usize] != kk {
                    self.pc += 2;
                }
            },
            0x5000 => {
                // 0x5XY0: Skip the next instruction if VX == VY
                //println!("{:X}\t|\tSE V[{:X}], V[{:X}]", self.pc, x, y);
                if self.v[x as usize] == self.v[y as usize] {
                    self.pc += 2;
                }
            },
            0x6000 => {
                // 0x6XKK: Set VX to KK
                //println!("{:X}\t|\tLD V[{:X}], {:02X}", self.pc, x, kk);
                self.v[x as usize] = kk;
            },
            0x7000 => {
                // 0x7XKK: Add KK to VX
                //println!("{:X}\t|\tADD V[{:X}], {:02X}", self.pc, x, kk);
                self.v[x as usize] += kk;
            },
            0x8000 => {
                match opcode & 0xF {
                    0x0 => {
                        // 0x8XY0: Set VX to VY
                        //println!("{:X}\t|\tLD V[{:X}], V[{:X}]", self.pc, x, y);
                        self.v[x as usize] = self.v[y as usize];
                    },
                    0x1 => {
                        // 0x8XY1: Set VX to VX | VY
                        //println!("{:X}\t|\tOR V[{:X}], V[{:X}]", self.pc, x, y);
                        self.v[x as usize] |= self.v[y as usize];
                        self.v[0xF] = 0;
                    },
                    0x2 => {
                        // 0x8XY2: Set VX to VX & VY
                        //println!("{:X}\t|\tAND V[{:X}], V[{:X}]", self.pc, x, y);
                        self.v[x as usize] &= self.v[y as usize];
                        self.v[0xF] = 0;
                    },
                    0x3 => {
                        // 0x8XY3: Set VX to VX ^ VY
                        //println!("{:X}\t|\tXOR V[{:X}], V[{:X}]", self.pc, x, y);
                        self.v[x as usize] ^= self.v[y as usize];
                        self.v[0xF] = 0;
                    },
                    0x4 => {
                        // 0x8XY4: Set VX to VX + VY
                        //println!("{:X}\t|\tADD V[{:X}], V[{:X}]", self.pc, x, y);
                        let a = self.v[x as usize] as u16;
                        let b = self.v[y as usize] as u16;
                        let sum = a + b;

                        let carry = sum > 0xFF;
                        self.v[x as usize] = sum as u8;

                        self.v[0xF] = 0;
                        if carry {
                            self.v[0xF] = 0x1;
                        }

                        //println!("0x8XY4: {} + {} = {}, {}", a, b, sum, self.v[0xF]);
                    },
                    0x5 => {
                        // 0x8XY5: Set VX to VX - VY
                        //println!("{:X}\t|\tSUB V[{:X}], V[{:X}]", self.pc, x, y);
                        let a = self.v[x as usize] as i16;
                        let b = self.v[y as usize] as i16;
                        let sub = a - b;

                        let carry = sub < 0;
                        self.v[x as usize] = (sub & 0xFF) as u8;

                        self.v[0xF] = 0x1;
                        if carry {
                            self.v[0xF] = 0x0;
                        }

                        //println!("0x8XY5: {} - {} = {}, {}", a, b, (sub & 0xFF) as u8, self.v[0xF]);
                    },
                    0x6 => {
                        // 0x8XY6: Set VF to the least significant bit of VX and shift VX to the right by 1
                        //println!("{:X}\t|\tSHR V[{:X}]", self.pc, x);
                        let f = self.v[x as usize] & 0x1;
                        self.v[x as usize] = self.v[y as usize] >> 1;
                        self.v[0xF] = f;
                    },
                    0x7 => {
                        // 0x8XY7: Set VX to VY - VX
                        //println!("{:X}\t|\tSUBN V[{:X}], V[{:X}]", self.pc, x, y);
                        let a = self.v[x as usize] as i16;
                        let b = self.v[y as usize] as i16;
                        let sub = b - a;

                        let carry = sub < 0x0;
                        self.v[x as usize] = (sub & 0xFF) as u8;

                        self.v[0xF] = 0x1;
                        if carry {
                            self.v[0xF] = 0x0;
                        }

                        //println!("0x8XY7: {} - {} = {}, {}", b, a, (sub & 0xFF) as u8, self.v[0xF]);
                    },
                    0xE => {
                        // 0x8XYE: Set VF to the most significant bit of VX and shift VX to the left by 1
                        let f = self.v[x as usize] >> 7;
                        self.v[x as usize] = self.v[y as usize] << 1;
                        self.v[0xF] = f;

                        //println!("0x8XYE: 0x{:02X} << 1 = 0x{:02X}, 0x{:02X}", f, self.v[x as usize], self.v[0xF]);
                    },
                    _ => println!("{:X}\t|\tUnknown opcode: {:X}", self.pc, opcode)
                }
            },
            0x9000 => {
                // 0x9XY0: Skip the next instruction if VX =! VY
                //println!("{:X}\t|\tSNE V[{:X}], V[{:X}]", self.pc, x, y);
                if self.v[x as usize] != self.v[y as usize] {
                    self.pc += 2;
                }
            },
            0xA000 => {
                // 0xANNN: Set I to NNN
                //println!("{:X}\t|\tLD I, {:03X}", self.pc, opcode & 0xFFF);
                self.i = opcode & 0xFFF;
            },
            0xB000 => {
                // 0xBNNN: Jump to NNN + V0
                //println!("{:X}\t|\tJP V0, {:03X}", self.pc, opcode & 0xFFF);
                self.pc = (opcode & 0xFFF) + self.v[0x0] as u16;
            },
            0xC000 => {
                // 0xCXKK: Set VX to a random number & KK
                //println!("{:X}\t|\tRND V[{:X}], {:02X}", self.pc, x, opcode & 0xFF);
                let rand: u8 = rand::thread_rng().gen_range(0..0xFF);
                self.v[x as usize] = rand & (opcode & 0xFF) as u8;
            },
            0xD000 => {
                // 0xDXYN: Draw sprite at VX, VY with a height of N
                //println!("{:X}\t|\tDRW V[{:X}], V[{:X}], {:X}", self.pc, x, y, opcode & 0xF);
                let width = 8;
                let height = (opcode & 0xF) as u8;

                self.v[0xF] = 0;

                for row in 0..height {
                    let mut sprite = self.memory[(self.i + row as u16) as usize];

                    for column in 0..width {

                        if (sprite & 0x80) > 0 {
                            let pixel_x = (self.v[x as usize] + column) & 63;
                            let pixel_y = (self.v[y as usize] + row) & 31;

                            if self.set_pixel(pixel_x, pixel_y) {
                                self.v[0xF] = 1;
                            }
                        }

                        sprite <<= 1;
                    }
                }
            },
            0xE000 => {
                match opcode & 0xFF {
                    0x9E => {
                        // 0xEX9E: Skip the next instruction if the key VX is pressed
                        //println!("{:X}\t|\tSKP V[{:X}]", self.pc, x);
                        if self.keys[self.v[x as usize] as usize] == 1 {
                            self.pc += 2;
                        }
                    },
                    0xA1 => {
                        // 0xEXA1: Skip the next instruction if the key VX is pressed
                        //println!("{:X}\t|\tSKNP V[{:X}]", self.pc, x);
                        if self.keys[self.v[x as usize] as usize] == 0 {
                            self.pc += 2;
                        }
                    },
                    _ => println!("{:X}\t|\tUnknown opcode: {:X}", self.pc, opcode)
                }
            },
            0xF000 => {
                match opcode & 0xFF {
                    0x07 => {
                        // FX07: Set VX to the value of the delay timer
                        //println!("{:X}\t|\tLD V[{:X}], DT", self.pc, x);
                        self.v[x as usize] = self.delay_timer;
                    },
                    0x0A => {
                        // FX0A: Pause until a key is pressed, then store the key in VX
                        //println!("{:X}\t|\tLD V[{:X}], K", self.pc, x);
                        self.paused = true;
                        self.waiting_for_key = true;
                    },
                    0x15 => {
                        // FX15: Set the delay timer to VX
                        //println!("{:X}\t|\tLD DT, V[{:X}]", self.pc, x);
                        self.delay_timer = self.v[x as usize];
                    },
                    0x18 => {
                        // FX18: Set the delay timer to VX
                        //println!("{:X}\t|\tLD ST, V[{:X}]", self.pc, x);
                        self.sound_timer = self.v[x as usize];
                    },
                    0x1E => {
                        // FX1E: Add I to VX
                        //println!("{:X}\t|\tADD I, V[{:X}]", self.pc, x);
                        self.i += self.v[x as usize] as u16;
                    },
                    0x29 => {
                        // FX29: Set I to the location of the sprite at VX
                        //println!("{:X}\t|\tLD F, V[{:X}]", self.pc, x);
                        self.i = self.v[x as usize] as u16;
                    },
                    0x33 => {
                        // FX33: Store the binary coded decimal representation of VX at the memory location starting at I
                        //println!("{:X}\t|\tLD B, V[{:X}]", self.pc, x);
                        let hundreds = self.v[x as usize] / 100;
                        let tens = (self.v[x as usize] / 10) % 10;
                        let ones = self.v[x as usize] % 10;

                        self.memory[self.i as usize] = hundreds;
                        self.memory[(self.i + 1) as usize] = tens;
                        self.memory[(self.i + 2) as usize] = ones;

                        //println!("{}: {} {} {}", self.v[x as usize], self.memory[self.i as usize], self.memory[(self.i + 1) as usize], self.memory[(self.i + 2) as usize]);
                    },
                    0x55 => {
                        // FX55: Copies memory from the addresses I to X to the registers V0 to VX
                        //println!("{:X}\t|\tLD [I], V[{:X}]", self.pc, x);
                        for register_index in 0..=x {
                            self.memory[(self.i + register_index as u16) as usize] = self.v[register_index as usize];
                        }
                    },
                    0x65 => {
                        // FX65: Copies memory from registers V0 to VX to the addresses I to X
                        //println!("{:X}\t|\tLD V[{:X}], [I]", self.pc, x);
                        for register_index in 0..=x {
                            self.v[register_index as usize] = self.memory[(self.i + register_index as u16) as usize];
                        }
                    },
                    _ => println!("{:X}\t|\tUnknown opcode: {:X}", self.pc, opcode)
                }
            },
            _ => println!("{:X}\t|\tUnknown opcode: {:X}", self.pc, opcode)
        }
    }
}