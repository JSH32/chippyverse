use std::{collections::HashMap, error::Error, fmt};

use once_cell::sync::Lazy;

use crate::types::{C8Addr, C8Byte, C8RegIdx};

/// Bad instruction.
#[derive(Debug)]
pub struct BadInstruction(pub String);

impl Error for BadInstruction {
    fn description(&self) -> &str {
        "bad instruction"
    }
}

impl fmt::Display for BadInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bad instruction: {}.", self.0)
    }
}

/// Opcode flag/mask.
type OpCodeFlagMask = (C8Addr, C8Addr);

/// Opcode enum.
#[derive(Debug, PartialEq)]
pub enum OpCode {
    /// 0nnn - SYS addr.
    /// * Jump to a machine code routine at nnn.
    ///
    /// | This instruction is only used on the old computers on which Chip-8 was
    /// | originally implemented.
    /// | It is ignored by modern interpreters.
    SYS(C8Addr),

    /// 00E0 - CLS.
    /// * Clear the display.
    CLS,

    /// 00EE - RET.
    /// * Return from a subroutine.
    ///
    /// | The interpreter sets the program counter to the address at the top
    /// | of the stack, then subtracts 1 from the stack pointer.
    RET,

    /// 1nnn - JP addr.
    /// * Jump to location nnn.
    ///
    /// | The interpreter sets the program counter to nnn.
    JP(C8Addr),

    /// 2nnn - CALL addr.
    /// * Call subroutine at nnn.
    ///
    /// | The interpreter increments the stack pointer,
    /// | then puts the current PC on the top of the stack.
    /// | The PC is then set to nnn.
    CALL(C8Addr),

    /// 3xkk - SE Vx, byte.
    /// * Skip next instruction if Vx = kk.
    ///
    /// | The interpreter compares register Vx to kk, and if they are equal,
    /// | increments the program counter by 2.
    SEByte(C8RegIdx, C8Byte),

    /// 4xkk - SNE Vx, byte.
    /// * Skip next instruction if Vx != kk.
    ///
    /// | The interpreter compares register Vx to kk, and if they are not equal,
    /// | increments the program counter by 2.
    SNEByte(C8RegIdx, C8Byte),

    /// 5xy0 - SE Vx, Vy.
    /// * Skip next instruction if Vx = Vy.
    ///
    /// | The interpreter compares register Vx to register Vy, and if they are
    /// | equal, increments the program counter by 2.
    SE(C8RegIdx, C8RegIdx),

    /// 6xkk - LD Vx, byte.
    /// * Set Vx = kk.
    ///
    /// | The interpreter puts the value kk into register Vx.
    LDByte(C8RegIdx, C8Byte),

    /// 7xkk - ADD Vx, byte.
    /// * Set Vx = Vx + kk.
    ///
    /// | Adds the value kk to the value of register Vx, then stores the
    /// | result in Vx.
    ADDByte(C8RegIdx, C8Byte),

    /// 8xy0 - LD Vx, Vy.
    /// * Set Vx = Vy.
    ///
    /// | Stores the value of register Vy in register Vx.
    LD(C8RegIdx, C8RegIdx),

    /// 8xy1 - OR Vx, Vy.
    /// * Set Vx = Vx OR Vy.
    ///
    /// | Performs a bitwise OR on the values of Vx and Vy, then stores the
    /// | result in Vx.
    /// | A bitwise OR compares the corrseponding bits from two values, and
    /// | if either bit is 1, then the same bit in the result is also 1.
    /// | Otherwise, it is 0.
    OR(C8RegIdx, C8RegIdx),

    /// 8xy2 - AND Vx, Vy.
    /// * Set Vx = Vx AND Vy.
    ///
    /// | Performs a bitwise AND on the values of Vx and Vy, then stores the
    /// | result in Vx.
    /// | A bitwise AND compares the corrseponding bits from two values, and if
    /// | both bits are 1, then the same bit in the result is also 1.
    /// | Otherwise, it is 0.
    AND(C8RegIdx, C8RegIdx),

    /// 8xy3 - XOR Vx, Vy.
    /// * Set Vx = Vx XOR Vy.
    ///
    /// | Performs a bitwise exclusive OR on the values of Vx and Vy, then
    /// | stores the result in Vx. An exclusive OR compares the corrseponding bits
    /// | from two values, and if the bits are not both the same, then the
    /// | corresponding bit in the result is set to 1.
    /// | Otherwise, it is 0.
    XOR(C8RegIdx, C8RegIdx),

    /// 8xy4 - ADD Vx, Vy.
    /// * Set Vx = Vx + Vy, set VF = carry.
    ///
    /// | The values of Vx and Vy are added together.
    /// | If the result is greater than 8 bits (i.e., > 255,) VF is set to 1,
    /// | otherwise 0.
    /// | Only the lowest 8 bits of the result are kept, and stored in Vx.
    ADD(C8RegIdx, C8RegIdx),

    /// 8xy5 - SUB Vx, Vy.
    /// * Set Vx = Vx - Vy, set VF = NOT borrow.
    ///
    /// | If Vx > Vy, then VF is set to 1, otherwise 0.
    /// | Then Vy is subtracted from Vx, and the results stored in Vx.
    SUB(C8RegIdx, C8RegIdx),

    /// 8xy6 - SHR Vx {, Vy}.
    /// * Set Vx = Vx SHR 1.
    ///
    /// | If the least-significant bit of Vx is 1, then VF is set to 1,
    /// | otherwise 0.
    /// | Then Vx is divided by 2.
    SHR(C8RegIdx, C8RegIdx),

    /// 8xy7 - SUBN Vx, Vy.
    /// * Set Vx = Vy - Vx, set VF = NOT borrow.
    ///
    /// | If Vy > Vx, then VF is set to 1, otherwise 0.
    /// | Then Vx is subtracted from Vy, and the results stored in Vx.
    SUBN(C8RegIdx, C8RegIdx),

    /// 8xyE - SHL Vx {, Vy}.
    /// * Set Vx = Vx SHL 1.
    ///
    /// | If the most-significant bit of Vx is 1, then VF is set to 1,
    /// | otherwise to 0.
    /// | Then Vx is multiplied by 2.
    SHL(C8RegIdx, C8RegIdx),

    /// 9xy0 - SNE Vx, Vy.
    /// * Skip next instruction if Vx != Vy.
    ///
    /// | The values of Vx and Vy are compared, and if they are not equal,
    /// | the program counter is increased by 2.
    SNE(C8RegIdx, C8RegIdx),

    /// Annn - LD I, addr.
    /// * Set I = nnn.
    ///
    /// | The value of register I is set to nnn.
    LDI(C8Addr),

    /// Bnnn - JP V0, addr.
    /// * Jump to location nnn + V0.
    ///
    /// | The program counter is set to nnn plus the value of V0.
    JP0(C8Addr),

    /// Cxkk - RND Vx, byte.
    /// * Set Vx = random byte AND kk.
    ///
    /// | The interpreter generates a random number from 0 to 255,
    /// | which is then ANDed with the value kk.
    /// | The results are stored in Vx.
    /// | See instruction 8xy2 for more information on AND.
    RND(C8RegIdx, C8Byte),

    /// Dxyn - DRW Vx, Vy, nibble.
    /// * Display n-byte sprite starting at memory location I at (Vx, Vy),
    ///   set VF = collision.
    ///
    /// | The interpreter reads n bytes from memory, starting at the address
    /// | stored in I.
    /// | These bytes are then displayed as sprites on screen at coordinates
    /// | (Vx, Vy).
    /// | Sprites are XORed onto the existing screen.
    /// | If this causes any pixels to be erased, VF is set to 1, otherwise it
    /// | is set to 0.
    /// | If the sprite is positioned so part of it is outside the coordinates
    /// | of the display, it wraps around to the opposite side of the screen.
    /// | See instruction 8xy3 for more information on XOR.
    DRW(C8RegIdx, C8RegIdx, C8Byte),

    /// Ex9E - SKP Vx.
    /// * Skip next instruction if key with the value of Vx is pressed.
    ///
    /// | Checks the keyboard, and if the key corresponding to the value of Vx
    /// | is currently in the down position, PC is increased by 2.
    SKP(C8RegIdx),

    /// ExA1 - SKNP Vx.
    /// * Skip next instruction if key with the value of Vx is not pressed.
    ///
    /// | Checks the keyboard, and if the key corresponding to the value of Vx
    /// | is currently in the up position, PC is increased by 2.
    SKNP(C8RegIdx),

    /// Fx07 - LD Vx, DT.
    /// * Set Vx = delay timer value.
    ///
    /// | The value of DT is placed into Vx.
    LDGetDelayTimer(C8RegIdx),

    /// Fx0A - LD Vx, K.
    /// * Wait for a key press, store the value of the key in Vx.
    ///
    /// | All execution stops until a key is pressed, then the value of that
    /// | key is stored in Vx.
    LDGetKey(C8RegIdx),

    /// Fx15 - LD DT, Vx.
    /// * Set delay timer = Vx.
    ///
    /// | DT is set equal to the value of Vx.
    LDSetDelayTimer(C8RegIdx),

    /// Fx18 - LD ST, Vx.
    /// * Set sound timer = Vx.
    ///
    /// | ST is set equal to the value of Vx.
    LDSetSoundTimer(C8RegIdx),

    /// Fx1E - ADD I, Vx.
    /// * Set I = I + Vx.
    ///
    /// | The values of I and Vx are added, and the results are stored in I.
    ADDI(C8RegIdx),

    /// Fx29 - LD F, Vx.
    /// * Set I = location of sprite for digit Vx.
    ///
    /// | The value of I is set to the location for the hexadecimal sprite
    /// | corresponding to the value of Vx.
    LDSprite(C8RegIdx),

    /// Fx33 - LD B, Vx.
    /// * Store BCD representation of Vx in memory locations I, I+1, and I+2.
    ///
    /// | The interpreter takes the decimal value of Vx, and places the hundreds
    /// | digit in memory at location in I, the tens digit at location I+1, and
    /// | the ones digit at location I+2.
    LDBCD(C8RegIdx),

    /// Fx55 - LD [I], Vx.
    /// * Store registers V0 through Vx in memory starting at location I.
    ///
    /// | The interpreter copies the values of registers V0 through Vx into
    /// | memory, starting at the address in I.
    LDS(C8RegIdx),

    /// Fx65 - LD Vx, [I].
    /// * Read registers V0 through Vx from memory starting at location I.
    ///
    /// | The interpreter reads values from memory starting at location I
    /// | into registers V0 through Vx.
    LDR(C8RegIdx),

    /// 0000 - EMPTY.
    EMPTY,

    /// xxxx - Data.
    DATA(C8Addr),
}

impl OpCode {
    /// Extract opcode ID.
    ///
    /// # Arguments
    ///
    /// * `opcode` - Opcode value.
    ///
    /// # Returns
    ///
    /// * Opcode address.
    ///
    fn extract_opcode_id(opcode: C8Addr) -> C8Addr {
        let mut extracted_key = None;

        for (key, flag_mask) in OPCODE_FLAG_MASKS.iter() {
            let flag = flag_mask.0;
            let mask = flag_mask.1;

            if mask & opcode == flag {
                extracted_key = Some(*key);
            }
        }

        if extracted_key.is_none() {
            extracted_key = Some(255)
        }

        extracted_key.unwrap()
    }

    /// Get opcode enum.
    ///
    /// # Arguments
    ///
    /// * `opcode` - Opcode value.
    ///
    /// # Returns
    ///
    /// * Opcode enum.
    ///
    pub fn from_opcode(opcode: C8Addr) -> Self {
        let action_id = Self::extract_opcode_id(opcode);

        let b3 = ((opcode & 0x0F00) >> 8) as C8Byte;
        let b2 = ((opcode & 0x00F0) >> 4) as C8Byte;
        let b1 = (opcode & 0x000F) as C8Byte;

        let addr = (C8Addr::from(b3) << 8) + (C8Addr::from(b2) << 4) + C8Addr::from(b1);
        let kk = (b2 << 4) + b1;

        match action_id {
            0 => Self::SYS(addr),
            1 => Self::CLS,
            2 => Self::RET,
            3 => Self::JP(addr),
            4 => Self::CALL(addr),
            5 => Self::SEByte(b3, kk),
            6 => Self::SNEByte(b3, kk),
            7 => Self::SE(b3, b2),
            8 => Self::LDByte(b3, kk),
            9 => Self::ADDByte(b3, kk),
            10 => Self::LD(b3, b2),
            11 => Self::OR(b3, b2),
            12 => Self::AND(b3, b2),
            13 => Self::XOR(b3, b2),
            14 => Self::ADD(b3, b2),
            15 => Self::SUB(b3, b2),
            16 => Self::SHR(b3, b2),
            17 => Self::SUBN(b3, b2),
            18 => Self::SHL(b3, b2),
            19 => Self::SNE(b3, b2),
            20 => Self::LDI(addr),
            21 => Self::JP0(addr),
            22 => Self::RND(b3, kk),
            23 => Self::DRW(b3, b2, b1),
            24 => Self::SKP(b3),
            25 => Self::SKNP(b3),
            26 => Self::LDGetDelayTimer(b3),
            27 => Self::LDGetKey(b3),
            28 => Self::LDSetDelayTimer(b3),
            29 => Self::LDSetSoundTimer(b3),
            30 => Self::ADDI(b3),
            31 => Self::LDSprite(b3),
            32 => Self::LDBCD(b3),
            33 => Self::LDS(b3),
            34 => Self::LDR(b3),
            35 => Self::EMPTY,
            _ => Self::DATA(opcode),
        }
    }

    /// Get string output for an opcode.
    /// Return a tuple: (assembly, verbose).
    ///
    /// # Arguments
    ///
    /// * `opcode` - Opcode enum.
    ///
    /// # Returns
    ///
    /// * String tuple (opcode, verbose opcode).
    ///
    pub fn get_opcode_str(&self) -> (String, String) {
        match self {
            Self::SYS(addr) => (format!("SYS {:04X}", addr), format!("executing system routine at {:04X} (NOP)", addr)),
            Self::CLS => ("CLS".into(), "clearing screen".into()),
            Self::RET => ("RET".into(), "return from subroutine".into()),
            Self::JP(addr) => (format!("JP {:04X}", addr), format!("jumping to address {:04X}", addr)),
            Self::CALL(addr) => (format!("CALL {:04X}", addr), format!("call subroutine at {:04X}", addr)),
            Self::SEByte(reg, byte) => (format!("SE V{:X}, {:02X}", reg, byte), format!("skip next instruction if V{:X} = {:02X}", reg, byte)),
            Self::SNEByte(reg, byte) => (format!("SNE V{:X}, {:02X}", reg, byte), format!("skip next instruction if V{:X} != {:02X}", reg, byte)),
            Self::SE(reg1, reg2) => (format!("SE V{:X}, V{:X}", reg1, reg2), format!("skip next instruction if V{:X} = V{:X}", reg1, reg2)),
            Self::LDByte(reg, byte) => (format!("LD V{:X}, {:02X}", reg, byte), format!("set V{:X} = {:02X}", reg, byte)),
            Self::ADDByte(reg, byte) => (format!("ADD V{:X}, {:02X}", reg, byte), format!("set V{:X} = V{:X} + {:02X}", reg, reg, byte)),
            Self::LD(reg1, reg2) => (format!("LD V{:X}, V{:X}", reg1, reg2), format!("set V{:X} = V{:X}", reg1, reg2)),
            Self::OR(reg1, reg2) => (format!("OR V{:X}, V{:X}", reg1, reg2), format!("set V{:X} = V{:X} OR V{:X}", reg1, reg1, reg2)),
            Self::AND(reg1, reg2) => (format!("AND V{:X}, V{:X}", reg1, reg2), format!("set V{:X} = V{:X} AND V{:X}", reg1, reg1, reg2)),
            Self::XOR(reg1, reg2) => (format!("XOR V{:X}, V{:X}", reg1, reg2), format!("set V{:X} = V{:X} XOR V{:X}", reg1, reg1, reg2)),
            Self::ADD(reg1, reg2) => (format!("AND V{:X}, V{:X}", reg1, reg2), format!("set V{:X} = V{:X} + V{:X}, set VF = carry", reg1, reg1, reg2)),
            Self::SUB(reg1, reg2) => (format!("SUB V{:X}, V{:X}", reg1, reg2), format!("set V{:X} = V{:X} - V{:X}, set VF = NOT borrow", reg1, reg1, reg2)),
            Self::SHR(reg, _) => (format!("SHR V{:X}", reg), format!("set V{:X} = V{:X} SHR 1", reg, reg)),
            Self::SUBN(reg1, reg2) => (format!("SUBN V{:X}, V{:X}", reg1, reg2), format!("set V{:X} = V{:X} - V{:X}, set VF = NOT borrow", reg1, reg2, reg1)),
            Self::SHL(reg, _) => (format!("SHL V{:X}", reg), format!("set V{:X} = V{:X} SHL 1", reg, reg)),
            Self::SNE(reg1, reg2) => (format!("SNE V{:X}, V{:X}", reg1, reg2), format!("skip next instruction if V{:X} != V{:X}", reg1, reg2)),
            Self::LDI(addr) => (format!("LD I, {:04X}", addr), format!("set I = {:04X}", addr)),
            Self::JP0(addr) => (format!("JP V0, {:04X}", addr), format!("jump to location {:04X} + V0", addr)),
            Self::RND(reg, byte) => (format!("RND V{:X}, {:02X}", reg, byte), format!("set V{:X} = random byte AND {:02X}", reg, byte)),
            Self::DRW(reg1, reg2, byte) => (format!("DRW V{:X}, V{:X}, {:02X}", reg1, reg2, byte), format!("display sprite starting at mem. location I at (V{:X}, V{:X}) on {} bytes, set VF = collision", reg1, reg2, byte)),
            Self::SKP(reg) => (format!("SKP V{:X}", reg), format!("skip next instruction if key with the value of V{:X} is pressed", reg)),
            Self::SKNP(reg) => (format!("SKNP V{:X}", reg), format!("skip next instruction if key with the value of V{:X} is not pressed", reg)),
            Self::LDGetDelayTimer(reg) => (format!("LD V{:X}, DT", reg), format!("set V{:X} = delay timer value", reg)),
            Self::LDGetKey(reg) => (format!("LD V{:X}, K", reg), format!("wait for a key press, store the value of the key in V{:X}", reg)),
            Self::LDSetDelayTimer(reg)
             => (format!("LD DT, V{:X}", reg), format!("set delay timer = V{:X}", reg)),
             Self::LDSetSoundTimer(reg) => (format!("LD ST, V{:X}", reg), format!("set sound timer = V{:X}", reg)),
             Self::ADDI(reg) => (format!("ADD I, V{:X}", reg), format!("set I = I + V{:X}", reg)),
            Self::LDSprite(reg) => (format!("LD F, V{:X}", reg), format!("set I = location of sprite for digit V{:X}", reg)),
            Self::LDBCD(reg) => (format!("LD B, V{:X}", reg), format!("store BCD representation of V{:X} in memory locations I, I+1 and I+2", reg)),
            Self::LDS(reg) => (format!("LD [I], V{:X}", reg), format!("store registers V0 through V{:X} in memory starting at location I", reg)),
            Self::LDR(reg) => (format!("LD V{:X}, [I]", reg), format!("read registers V0 through V{:X} from memory starting at location I", reg)),
            Self::EMPTY => ("EMPTY".into(), "- empty".into()),
            Self::DATA(opcode) => (format!("DATA {:04X}", opcode), format!("- data ({:04X})", opcode))
        }
    }
}

static OPCODE_FLAG_MASKS: Lazy<HashMap<C8Addr, OpCodeFlagMask>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert(0, (0x0FFF, 0x0000)); // 0nnn
    m.insert(1, (0x00E0, 0xFFFF)); // 00E0
    m.insert(2, (0x00EE, 0xFFFF)); // 00EE
    m.insert(3, (0x1000, 0xF000)); // 1nnn
    m.insert(4, (0x2000, 0xF000)); // 2nnn
    m.insert(5, (0x3000, 0xF000)); // 3xkk
    m.insert(6, (0x4000, 0xF000)); // 4xkk
    m.insert(7, (0x5000, 0xF00F)); // 5xy0
    m.insert(8, (0x6000, 0xF000)); // 6xkk
    m.insert(9, (0x7000, 0xF000)); // 7xkk
    m.insert(10, (0x8000, 0xF00F)); // 8xy0
    m.insert(11, (0x8001, 0xF00F)); // 8xy1
    m.insert(12, (0x8002, 0xF00F)); // 8xy2
    m.insert(13, (0x8003, 0xF00F)); // 8xy3
    m.insert(14, (0x8004, 0xF00F)); // 8xy4
    m.insert(15, (0x8005, 0xF00F)); // 8xy5
    m.insert(16, (0x8006, 0xF00F)); // 8xy6
    m.insert(17, (0x8007, 0xF00F)); // 8xy7
    m.insert(18, (0x800E, 0xF00F)); // 8xyE
    m.insert(19, (0x9000, 0xF00F)); // 9xy0
    m.insert(20, (0xA000, 0xF000)); // Annn
    m.insert(21, (0xB000, 0xF000)); // Bnnn
    m.insert(22, (0xC000, 0xF000)); // Cxkk
    m.insert(23, (0xD000, 0xF000)); // Dxyn
    m.insert(24, (0xE09E, 0xF0FF)); // Ex9E
    m.insert(25, (0xE0A1, 0xF0FF)); // ExA1
    m.insert(26, (0xF007, 0xF0FF)); // Fx07
    m.insert(27, (0xF00A, 0xF0FF)); // Fx0A
    m.insert(28, (0xF015, 0xF0FF)); // Fx15
    m.insert(29, (0xF018, 0xF0FF)); // Fx18
    m.insert(30, (0xF01E, 0xF0FF)); // Fx1E
    m.insert(31, (0xF029, 0xF0FF)); // Fx29
    m.insert(32, (0xF033, 0xF0FF)); // Fx33
    m.insert(33, (0xF055, 0xF0FF)); // Fx55
    m.insert(34, (0xF065, 0xF0FF)); // Fx65

    m.insert(35, (0x0000, 0xFFFF)); // 0000

    m
});

/// Extract opcode from array.
///
/// # Arguments
///
/// * `array` - Array.
/// * `ptr` - Pointer.
///
/// # Returns
///
/// * Opcode address.
///
pub fn extract_opcode_from_array(array: &[u8], ptr: usize) -> C8Addr {
    let array_length = array.len();

    if ptr >= array_length || (ptr + 1) >= array_length {
        // Return 0 if the opcode is not complete.
        0
    } else {
        (C8Addr::from(array[ptr]) << 8) + C8Addr::from(array[ptr + 1])
    }
}
