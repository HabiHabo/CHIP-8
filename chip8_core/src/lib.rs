use rand::random;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

const START_ADDR: u16 = 0x200;

impl Default for  Emu {
    fn default() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }
}

impl Emu {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    pub fn tick(&mut self) {
        // Fetch
        let op = self.fetch();
        // Decode and Execute
        self.execute(op);
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => {return;}, //Do nothing
            (0, 0, 0xE, 0) => {self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];}, //Reset screen
            (0, 0, 0xE, 0xE) => {self.pc = self.pop()}, //Return from a subroutine
            (0x1, _, _, _) => {self.pc = op & 0x0FFF},
            (0x2, _, _, _) => { //Enter a subroutine
                self.push(self.pc);
                self.pc = op & 0x0FFF;
            },
            (0x3, _, _, _) => { //Skip if VX == NN
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2; //Move forward 2 bytes, which is length of one instruction
                }
            },
            (0x4, _, _, _) => { //Skip if VX != NN
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            },
            (0x5, _, _, 0) => { //Skip if VX == VY
                let x = digit2 as usize;
                let y = digit3 as usize;
                if (self.v_reg[x] == self.v_reg[y]) {
                    self.pc += 2;
                }
            },
            (0x6, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = nn;
            },
            (0x7, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn);
            }
            (0x8, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y]
            },
            (0x8, _, _, 0x1) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y]
            },
            (0x8, _, _, 0x2) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y]
            },
            (0x8, _, _, 0x3) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y]
            },
            (0x8, _, _, 0x4) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, overflowbool) = self.v_reg[x].overflowing_add(self.v_reg[y]);

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = if overflowbool { 1 } else { 0 };
            },
            (0x8, _, _, 0x5) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, underflowbool) = self.v_reg[x].overflowing_sub(self.v_reg[y]);

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = if underflowbool { 0 } else { 1 };
            },
            (0x8, _, _, 0x6) => {
                let x = digit2 as usize;
                let z = self.v_reg[x] & 0x1;

                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = z;

            },
            (0x8, _, _, 0x7) => {
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, underflowbool) = self.v_reg[y].overflowing_sub(self.v_reg[x]);

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = if underflowbool { 0 } else { 1 };
            },
            (0x8, _, _, 0xE) => {
                let x = digit2 as usize;
                let z = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = z;
            },
            (0x9, _, _, 0) => {
                let x = digit2 as usize;
                let y = digit3 as usize;
                if (self.v_reg[x] != self.v_reg[y]) {
                    self.pc += 2;
                }
            },
            (0xA, _, _, _) => {
                let nnn = (op & 0x0FFF) as u16;
                self.i_reg = nnn
            }
            (0xB, _, _, _) => {
                let nnn = (op & 0x0FFF) as u16;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }
            (0xC, _, _, _) => {
                let x = digit2 as usize;
                let nn = (op & 0x00FF) as u8;
                let rng: u8 = random();
                self.v_reg[x] = rng & nn;
            }
            (0xD, _, _, _) => {
                let x_cord = self.v_reg[digit2 as usize] as u16;
                let y_cord = self.v_reg[digit3 as usize] as u16;
                let height = digit4;

                let mut flipped = false;

                for row in 0..height {
                    let adress = self.i_reg + row as u16;
                    let pixels = self.ram[adress as usize];
                    for col in 0..8 {
                        if (pixels & 0b1000_0000 >> col) != 0 {
                            let x = (x_cord + col) as usize % SCREEN_WIDTH;
                            let y = (y_cord + row) as usize % SCREEN_HEIGHT;

                            let pixel_id = x + SCREEN_WIDTH * y;
                            flipped |= self.screen[pixel_id]; //Bitwise OR
                            self.screen[pixel_id] ^= true; //Bitwise XOR
                        }
                    }
                }


                if flipped {
                    self.v_reg[0xF] = 1;
                } else {
                    self.v_reg[0xF] = 0;
                }
            },
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if key {
                    self.pc += 2;
                }
            },
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];
                let key = self.keys[vx as usize];
                if !key {
                    self.pc += 2;
                }
            },
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            },
            (0xF, _, 0, 0xA) => {
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }
                if !pressed {
                    self.pc -= 2;
                }
            },
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            },
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            },
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                self.i_reg = self.i_reg.wrapping_add(self.v_reg[x] as u16);
            },
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5;
            },
            (0xF, _, 3, 3) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundred = (vx / 100.0).floor() as u8;
                let ten = ((vx / 10.0) % 10.0).floor() as u8;
                let one  = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundred;
                self.ram[(self.i_reg + 1) as usize] = ten;
                self.ram[(self.i_reg + 2) as usize] = one;
            },
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let vi = self.i_reg as usize;

                for v_iterate in 0..=x {
                    self.ram[vi + v_iterate] = self.v_reg[v_iterate];
                }
            },
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let vi = self.i_reg as usize;

                for v_iterate in 0..=x {
                    self.v_reg[v_iterate] = self.ram[vi + v_iterate];
                }
            }


            (_, _, _, _) => unimplemented!("Unimplemented opcode: {}", op),
        }


    }


    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;
        op
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if (self.st == 1){
                //TODO Sound
            }
            self.st -= 1;
        }
    }
}


