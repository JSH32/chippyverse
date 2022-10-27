mod interpreter;
pub mod keypad;
pub mod opcode;
pub mod types;

use std::{
    ops::Deref,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    thread::{self},
    time::{Duration, Instant},
};

pub use keypad::Keypad;
use types::C8Byte;

/// Chip8 emulator with both JIT and interpreter.
/// Members are only public for debugging purposes.
pub struct Chip8 {
    // Program counter, first 200 bits reserved.
    pub pc: u16,
    // Stack pointer.
    pub sp: usize,
    // Current index, 12 bit.
    pub index: u16,
    // 4 kilobyte memory.
    pub memory: [C8Byte; 4096],
    // Stack of size 16 with 16-bit values.
    pub stack: [u16; 16],
    // 16 8-bit registers (VX) where X is 0-F
    pub registers: [u8; 16],
    // Delay timer, counts down independently of clock speed.
    pub delay_timer: u8,
    // Sound timer, counts down while beeping until 0.
    pub sound_timer: u8,

    // Video memory, 64 height, 32 length
    pub screen: [[bool; 64]; 32],

    pub keypad: Keypad,
    timer: Instant,
}

/// Create a shared chip8 executing on its own thread.
pub struct ExecutingChip8 {
    chip8: Arc<RwLock<Chip8>>,
    // join_handle: JoinHandle<Thread>,
    running: Arc<AtomicBool>,
}

impl Deref for ExecutingChip8 {
    type Target = Arc<RwLock<Chip8>>;

    fn deref(&self) -> &Self::Target {
        &self.chip8
    }
}

impl ExecutingChip8 {
    pub fn new() -> Self {
        let chip8 = Arc::new(RwLock::new(Chip8::new()));
        let running = Arc::new(AtomicBool::new(false));

        let chip8_clone = chip8.clone();
        let running_clone = running.clone();
        thread::spawn(move || {
            loop {
                // Wait while running is disabled.
                while !running_clone.load(Ordering::Relaxed) {}

                let init_time = Instant::now();

                chip8_clone.write().unwrap().interpreter();

                let end_time = Instant::now();

                // Wait here til time for more cycles
                while Instant::now()
                    < end_time + Duration::from_nanos(1000000000 / 600) - (end_time - init_time)
                {
                }
            }
        });

        Self { chip8, running }
    }

    /// Should the managed thread be executing.
    pub fn set_running(&self, start: bool) {
        self.running.store(start, Ordering::Relaxed)
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }
}

impl Chip8 {
    /// Create a chip8 emulator without a driving thread.
    pub fn new() -> Self {
        let mut state = Self {
            pc: 0x200, // First 200 bits reserved usually
            sp: 0,
            index: 0,
            memory: [0; 4096],
            stack: [0; 16],
            registers: [0; 16],
            delay_timer: 0,
            sound_timer: 0,
            screen: [[false; 64]; 32],
            timer: Instant::now(),
            keypad: Keypad::default(),
        };

        state.load_font();
        state
    }

    /// Reset the state of the emulator.
    pub fn reset_state(&mut self) {
        self.delay_timer = 0;
        self.sound_timer = 0;
        self.pc = 0x200;
        self.index = 0;
        self.stack.fill(0);
        self.registers.fill(0);
        self.load_font();
        self.clear_screen();
    }

    /// Load rom into memory.
    /// This will call `reset_state`
    pub fn load_rom(&mut self, rom: Vec<u8>) {
        self.reset_state();

        let mut rom = rom.clone();
        rom.resize(4096 - 512, 0);
        self.memory[512..4096].copy_from_slice(&rom)
    }

    /// Loaded font to the first 80 bytes of memory
    fn load_font(&mut self) {
        self.memory[0..80].copy_from_slice(&[
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
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ]);
    }

    /// Clear all video memory.
    fn clear_screen(&mut self) {
        self.screen = [[false; 64]; 32];
    }

    // #[cfg(target_os = "windows")]
    fn timer(&mut self) {
        if self.timer.elapsed() >= Duration::from_micros(16666) {
            if self.delay_timer > 0 {
                self.delay_timer -= 1;
            }

            if self.sound_timer > 0 {
                // TODO: Accept sound callback when start/stop playing sound.
                self.sound_timer -= 1;
            }

            self.timer = Instant::now();
        }
    }

    // Draw sprite at coordinates to video memory.
    // This also sets the carry register.
    fn draw_sprite(&mut self, x: usize, y: usize, n: u8) {
        for j in 0..n {
            let line = self.memory[(self.index + j as u16) as usize];

            for i in 0..8 {
                if line & (0x80 >> i) != 0 {
                    let y = ((self.registers[y] + j) % 32) as usize;
                    let x = ((self.registers[x] + i) % 64) as usize;

                    if self.screen[y][x] {
                        self.screen[y][x] = false;
                        self.registers[15] = 1;
                    } else {
                        self.screen[y][x] = true;
                        self.registers[15] = 0;
                    }
                }
            }
        }
    }
}

/// Get any value as a pointer or memory address for JIT access.
trait Address {
    fn address(&self, offset: usize) -> usize;
}

impl<T> Address for T {
    fn address(&self, offset: usize) -> usize {
        self as *const T as usize + offset
    }
}
