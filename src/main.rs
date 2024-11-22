use std::io::Write;
use std::{fs};
use std::ops::{Div, Mul};
use std::process::exit;
use std::time::{Duration, Instant};
use macroquad::prelude::*;
use clap::Parser;

struct ChipP {
    registers: [u16; 16],
    pc: u32,
    steps_since_start: u32,
    sp: u16,
    stack: [u32; 1024],
    rom: Vec<u8>,
    rom_size: usize,
    memory: [u8; 65536],
    display_buffers: [[[u32; 480]; 640]; 2], // display_buffers[current_buffer][y][x] = color (RGBA8888)
    current_buffer: u8,
    opcode: u8,
}

impl ChipP {
    fn new() -> Self {
        Self {
            registers: [0; 16],
            pc: 0,
            steps_since_start: 0,
            sp: 0,
            stack: [0; 1024],
            rom: Vec::new(),
            rom_size: 0,
            memory: [0; 65536],
            display_buffers: [[[0; 480]; 640]; 2],
            current_buffer: 0,
            opcode: 0,
        }
    }

    pub fn load_rom(&mut self, rom_path: String){
        let data = fs::read(rom_path).unwrap();
        if data.len() as usize > 65536*65536 {
            panic!("ROM file is too large!");
        }
        self.rom = data;
        self.rom_size = self.rom.len();
    }

    pub fn load_rom_raw(&mut self, rom: &[u8]) {
        self.rom = rom.to_vec();
        self.rom_size = self.rom.len();
    }

    pub fn get8rom(&mut self) -> u8 {
        let out = self.rom[(self.pc) as usize];
        self.pc = self.pc.wrapping_add(1);
        out
    }

    pub fn get16rom(&mut self) -> u16 {
        let out = ((self.rom[(self.pc) as usize] as u16) << 8) | self.rom[(self.pc+1) as usize] as u16;
        self.pc = self.pc.wrapping_add(2);
        out
    }

    #[allow(dead_code)]
    pub fn get32rom(&mut self) -> u32 {
        let out = ((self.rom[(self.pc) as usize] as u32) << 24) |
            ((self.rom[(self.pc+1) as usize] as u32) << 16) |
            ((self.rom[(self.pc+2) as usize] as u32) << 8) |
            self.rom[(self.pc+3) as usize] as u32;

        self.pc = self.pc.wrapping_add(4);
        out
    }

    #[allow(dead_code)]
    pub fn get8(&mut self, addr: u16) -> u8 {
        self.memory[addr as usize]
    }

    #[allow(dead_code)]
    pub fn get16(&mut self, addr: u16) -> u16 {
        ((self.memory[addr as usize] as u16) << 8) | self.memory[(addr+1) as usize] as u16
    }

    #[allow(dead_code)]
    pub fn get32(&mut self, addr: u16) -> u32 {
        ((self.memory[addr as usize] as u32) << 24) |
            ((self.memory[(addr+1) as usize] as u32) << 16) |
            ((self.memory[(addr+2) as usize] as u32) << 8) |
            self.memory[(addr+3) as usize] as u32
    }

    pub fn draw_buffer(&mut self){
        for x in 0..self.display_buffers[0][0].len() {
            for y in 0..self.display_buffers[0].len() {
                draw_rectangle(x as f32, y as f32, 1f32, 1f32, Color::from_hex(
                    self.display_buffers[self.current_buffer as usize][y][x]
                ));
            }
        }
    }

    pub fn step(&mut self) -> bool {
        if(self.pc >= self.rom_size as u32) {
            return false;
            }
        self.opcode = self.get8rom();
        match self.opcode {
            0x00 => {self.opcode -= 1},
            0x01 => self.op_mov(),
            0x02 => self.op_store_i(),
            0x03 => self.op_load_i(),
            0x04 => self.op_store(),
            0x05 => self.op_load(),
            0x06 => self.op_add(),
            0x07 => self.op_add_i(),
            0x08 => self.op_sub(),
            0x09 => self.op_sub_i(),
            0x0A => self.op_mul(),
            0x0B => self.op_div(),
            0x0C => self.op_jmp(),
            0x0D => self.op_jeq(),
            0x0E => self.op_jne(),
            0x0F => self.op_store_str(),
            0x10 => self.op_print_str_mem(),
            0x11 => self.op_print_str_rom(),
            0x12 => self.op_call(),
            0x13 => self.op_return(),
            0x14 => self.op_swap_buffers(),
            0x15 => self.op_draw_pixel(),
            0xFF => return false,
            _ => {}
        }
        self.steps_since_start = self.steps_since_start.wrapping_add(1);
        true
    }

    // load an immediate (val2) into register (val1)
    pub fn op_mov(&mut self) {
        let reg = self.get8rom();
        let val = self.get16rom();
        self.registers[reg as usize] = val;
    }

    // store reg1 (val1) into memory at addr - addr+1 (val2)
    pub fn op_store_i(&mut self) {
        let reg = self.get8rom();
        let addr = self.get16rom();
        self.memory[addr as usize] = (self.registers[reg as usize] >> 8) as u8;
        self.memory[(addr+1) as usize] = (self.registers[reg as usize] & 0xff) as u8;
    }

    // load reg1 (val1) using memory at addr - addr+1 (val2)
    pub fn op_load_i(&mut self) {
        let reg = self.get8rom();
        let addr = self.get16rom();
        self.registers[reg as usize] = self.get16(addr);
    }

    // store reg1 (val1) into memory at the address specified by reg2 (val2)
    pub fn op_store(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let addr = self.registers[reg2 as usize];
        self.memory[addr as usize] = (self.registers[reg1 as usize] >> 8) as u8;
        self.memory[(addr+1) as usize] = (self.registers[reg1 as usize] & 0xff) as u8;
    }

    // load reg1 (val1) using memory at the address specified by reg2 (val2)
    pub fn op_load(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let addr = self.registers[reg2 as usize];
        self.registers[reg1 as usize] = self.get16(addr);
    }

    // set reg1 (val1) to reg1 + reg2 (val2)
    pub fn op_add(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_add(self.registers[reg2 as usize]);
    }

    // set reg1 (val1) to reg1 + immediate (val2)
    pub fn op_add_i(&mut self) {
        let reg1 = self.get8rom();
        let val = self.get16rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_add(val);
    }

    // set reg1 (val1) to reg1 - reg2 (val2)
    pub fn op_sub(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_sub(self.registers[reg2 as usize]);
    }

    // set reg1 (val1) to reg1 - immediate (val2)
    pub fn op_sub_i(&mut self) {
        let reg1 = self.get8rom();
        let val = self.get16rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_sub(val);
    }

    // set reg1 (val1) to reg1 * reg2 (val2)
    pub fn op_mul(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].mul(self.registers[reg2 as usize]);
    }

    // set reg1 (val1) to reg1 / reg2 (val2)
    pub fn op_div(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].div(self.registers[reg2 as usize]);
    }

    pub fn jump(&mut self, page: u16, offset: u16){
        self.pc = (page * 65535) as u32 + offset as u32;
    }

    // unconditional jump to addr (val1)
    pub fn op_jmp(&mut self) {
        let page = self.get16rom();
        let offset = self.get16rom();
        self.jump(page, offset);
    }

    // jump to addr (val3) if reg1 (val1) equals reg2 (val2)
    pub fn op_jeq(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let page = self.get16rom();
        let offset = self.get16rom();
        if self.registers[reg1 as usize] == self.registers[reg2 as usize] {
            self.jump(page, offset);
        }
    }

    // jump to addr (val3) if reg1 (val1) does not equal reg2 (val2)
    pub fn op_jne(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let page = self.get16rom();
        let offset = self.get16rom();
        if self.registers[reg1 as usize] != self.registers[reg2 as usize] {
            self.jump(page, offset);
        }
    }

    // store a null terminated string located in rom at addr (val1) into memory starting at addr (val2)
    pub fn op_store_str(&mut self){
        let mut addr1 = self.get32rom();
        let mut addr2 = self.get16rom();
        let mut c = self.rom[addr1 as usize];
        while c != 0x00 {
            self.memory[addr2 as usize] = c;
            addr1 += 1;
            addr2 += 1;
            c = self.rom[addr1 as usize];
        }
        self.memory[addr2 as usize] = c;
    }

    // print a null terminated string from memory starting at addr (val1)
    pub fn op_print_str_mem(&mut self){
        let mut addr = self.get16rom();
        let mut c = self.get8(addr);
        while c != 0x00 {
            print!("{}", c as char);
            addr += 1;
            c = self.get8(addr);
        }
        std::io::stdout().flush().unwrap();
    }

    // print a null terminated string from rom starting at addr (val1)
    pub fn op_print_str_rom(&mut self){
        let mut addr = self.get32rom();
        let mut c = self.rom[(addr) as usize];
        while c != 0x00 {
            print!("{}", c as char);
            addr += 1;
            c = self.rom[(addr) as usize];
        }
        std::io::stdout().flush().unwrap();
    }

    // push current pc to stack and jump to addr (val1)
    pub fn op_call(&mut self) {
        let page = self.get16rom();
        let offset = self.get16rom();
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        if self.sp > self.stack.len() as u16 {
            println!("Stack overflow");
            exit(1);
        }
        self.jump(page, offset);
    }

    // pop top of stack to pc
    pub fn op_return(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    pub fn op_swap_buffers(&mut self){
        if self.current_buffer == 0 {
            self.current_buffer = 1;
        } else {
            self.current_buffer = 0;
        }
    }

    pub fn op_draw_pixel(&mut self) {
        let cb;
        if self.current_buffer == 0 {
            cb = 1usize;
        } else {
            cb = 0usize;
        }
        let x_pos = self.registers[self.get8rom() as usize] as usize % self.display_buffers[0][0].iter().len();
        let y_pos = self.registers[self.get8rom() as usize] as usize % self.display_buffers[0].iter().len();
        let color = self.get32rom();
        self.display_buffers[cb][y_pos][x_pos] = color;
    }

    // pub fn op_draw_sprite(&mut self) {
    //     let x_pos = self.registers[self.get8rom() as usize] as usize % self.display_buffers[0][0].len();
    //     let y_pos = self.registers[self.get8rom() as usize] as usize % self.display_buffers[0].len();
    //     let start_addr = self.get32rom();
    // }
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    rom: Option<String>,
}

#[macroquad::main("MyGame")]
async fn main() {
    let args = Args::parse();
    let rom_path = args.rom.unwrap_or("res/out.p16".to_string());
    let mut state = Box::new(ChipP::new());
    state.load_rom(rom_path);
    // clear_background(BLACK);
    // let array: Vec<usize> = (0..state.rom_size).collect();
    // println!("{:02X?}", array);
    // println!("{:02X?}", state.rom);
    // println!("{:#04X}, {:#04X}", state.pc, state.rom[0]);
    let mut last_step = Instant::now();
    let mut last_frame = Instant::now();
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        state.step();
        if last_step.elapsed().as_micros() > 500u128 {
            last_step = std::time::Instant::now();
        }

        if last_frame.elapsed().as_secs_f32() > 1f32 / 60f32 {
            last_frame = std::time::Instant::now();
            next_frame().await;
            state.draw_buffer();
        }
    }
}