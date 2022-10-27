use crate::opcode::{extract_opcode_from_array, OpCode};
use crate::types::C8Addr;
use crate::Chip8;

use rand::Rng;

impl Chip8 {
    /// Executes a single instruction using the interpreter.
    pub fn interpreter(&mut self) {
        // Should this advance the program counter by 2
        let mut advance_pointer = true;

        let opcode = OpCode::from_opcode(extract_opcode_from_array(&self.memory, self.pc as usize));

        match opcode {
            OpCode::CLS => self.clear_screen(),
            OpCode::RET => {
                if self.sp > 0 {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                } else {
                    println!("Stack underflow (RET 0x00EE)");
                }
            }
            OpCode::JP(addr) => {
                self.pc = addr;
                advance_pointer = false;
            }
            OpCode::CALL(addr) => {
                if self.sp < 15 {
                    self.stack[self.sp as usize] = self.pc;
                    self.sp += 1;
                    self.pc = addr;
                    advance_pointer = false;
                } else {
                    println!("Stack overflow (CALL 0x2nnn)");
                }
            }
            OpCode::SEByte(reg, byte) => {
                if self.registers[reg as usize] == byte {
                    self.pc += 2;
                }
            }
            OpCode::SNEByte(reg, byte) => {
                if self.registers[reg as usize] != byte {
                    self.pc += 2;
                }
            }
            OpCode::SE(reg1, reg2) => {
                if self.registers[reg1 as usize] == self.registers[reg2 as usize] {
                    self.pc += 2;
                }
            }
            OpCode::LDByte(reg, byte) => {
                self.registers[reg as usize] = byte;
            }
            OpCode::ADDByte(reg, byte) => {
                self.registers[reg as usize] = self.registers[reg as usize].wrapping_add(byte);
            }
            OpCode::LD(reg1, reg2) => {
                self.registers[reg1 as usize] = self.registers[reg2 as usize];
            }
            OpCode::OR(reg1, reg2) => {
                self.registers[reg1 as usize] |= self.registers[reg2 as usize];
            }
            OpCode::AND(reg1, reg2) => {
                self.registers[reg1 as usize] &= self.registers[reg2 as usize];
            }
            OpCode::XOR(reg1, reg2) => {
                self.registers[reg1 as usize] ^= self.registers[reg2 as usize];
            }
            OpCode::ADD(reg1, reg2) => {
                let r1 = self.registers[reg1 as usize];
                let r2 = self.registers[reg2 as usize];
                let (res, overflow) = r1.overflowing_add(r2);

                // Check if overflow.
                if overflow {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                }

                self.registers[reg1 as usize] = res;
            }
            OpCode::SUB(reg1, reg2) => {
                let r1 = self.registers[reg1 as usize];
                let r2 = self.registers[reg2 as usize];
                let res = r1.wrapping_sub(r2);

                if r1 > r2 {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                }

                self.registers[reg1 as usize] = res;
            }
            OpCode::SHR(reg, _) => {
                let r = self.registers[reg as usize];

                if r & 1 == 1 {
                    self.registers[15] = 1
                } else {
                    self.registers[15] = 0;
                }

                self.registers[reg as usize] = r >> 1;
            }
            OpCode::SUBN(reg1, reg2) => {
                let r1 = self.registers[reg1 as usize];
                let r2 = self.registers[reg2 as usize];
                let res = r2.wrapping_sub(r1);

                if r2 > r1 {
                    self.registers[15] = 1
                } else {
                    self.registers[15] = 0;
                }

                self.registers[reg1 as usize] = res;
            }
            OpCode::SHL(reg, _) => {
                let r = self.registers[reg as usize];
                let msb = 1 << 7;

                if r & msb == msb {
                    self.registers[15] = 1;
                } else {
                    self.registers[15] = 0;
                }

                self.registers[reg as usize] = r << 1;
            }
            OpCode::SNE(reg1, reg2) => {
                let r1 = self.registers[reg1 as usize];
                let r2 = self.registers[reg2 as usize];

                if r1 != r2 {
                    self.pc += 2;
                }
            }
            OpCode::LDI(addr) => self.index = addr,
            OpCode::JP0(addr) => self.pc = addr + self.registers[0] as C8Addr,
            OpCode::RND(reg, byte) => {
                self.registers[reg as usize] = rand::thread_rng().gen_range(0..256) as u8 & byte;
            }
            OpCode::DRW(reg1, reg2, byte) => {
                self.draw_sprite(reg1 as usize, reg2 as usize, byte);
            }
            OpCode::SKP(reg) => {
                if self.keypad.keys[self.registers[reg as usize] as usize] {
                    self.pc += 2;
                }
            }
            OpCode::SKNP(reg) => {
                if !self.keypad.keys[self.registers[reg as usize] as usize] {
                    self.pc += 2;
                }
            }
            OpCode::LDGetDelayTimer(reg) => {
                self.registers[reg as usize] = self.delay_timer;
            }
            OpCode::LDGetKey(reg) => {
                let mut pressed = false;
                for i in 0..15 {
                    if self.keypad.keys[i as usize] {
                        self.registers[reg as usize] = i as u8;
                        pressed = true;
                    }
                }

                if !pressed {
                    // We don't want to iterate until key was pressed.
                    advance_pointer = false;
                }
            }
            OpCode::LDSetDelayTimer(reg) => {
                self.delay_timer = self.registers[reg as usize];
            }
            OpCode::LDSetSoundTimer(reg) => {
                self.sound_timer = self.registers[reg as usize];
            }
            OpCode::ADDI(reg) => {
                self.index += self.registers[reg as usize] as C8Addr;
            }
            OpCode::LDSprite(reg) => {
                self.index = self.registers[reg as usize] as u16 * 5;
            }
            OpCode::LDBCD(reg) => {
                // let x = opcode.x();
                let reg = self.registers[reg as usize];

                self.memory[self.index as usize] = reg / 100;
                self.memory[(self.index + 1) as usize] =
                    (reg - self.memory[self.index as usize] * 100) / 10;
                self.memory[(self.index + 2) as usize] = (reg
                    - self.memory[self.index as usize] * 100)
                    - self.memory[(self.index + 1) as usize] * 10;
            }
            OpCode::LDS(reg) => {
                for i in 0..=reg as usize {
                    self.memory[self.index as usize + i] = self.registers[i];
                }
            }
            OpCode::LDR(reg) => {
                for i in 0..=reg as usize {
                    self.registers[i] = self.memory[self.index as usize + i];
                }
            }
            _ => {
                // The rest are treated as NOP
            }
        };

        if advance_pointer {
            self.pc += 2;
        }

        self.timer();
    }
}
