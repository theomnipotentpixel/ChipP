use std::io::Write;
use std::{fs};
use std::collections::HashMap;
use std::ops::{Div, Mul};
use std::process::exit;
use std::time::{Instant};
use macroquad::prelude::*;
use clap::Parser;
use macroquad::texture::Texture2D;

struct ChipP {
    registers: [u32; 32],
    pc: u32,
    steps_since_start: u32,
    sp: u16,
    stack: Vec<u32>,
    rom: Vec<u8>,
    rom_size: usize,
    memory: Vec<u8>,
    display_buffers: [RenderTarget; 2], // display_buffers[current_buffer][y][x] = color (RGBA8888)
    current_buffer: usize,
    opcode: u8,
    sprites: HashMap<u32, Texture2D>,
    display_size: [usize; 2],
    cameras: [Camera2D; 2]
}

impl ChipP {
    fn new() -> Self {
        let mut stack: Vec<u32> = Vec::with_capacity(1024);
        stack.fill(0);
        let mut memory: Vec<u8> = Vec::with_capacity(65535);
        memory.fill(0);
        let display_size = [320, 240];

        let display_buffers = [
            render_target(display_size[0], display_size[1]),
            render_target(display_size[0], display_size[1])
        ];

        let cam_zoom = vec2(1f32 / display_size[0] as f32, 1f32 / display_size[1] as f32);
        let cam_target = vec2(display_size[0] as f32, display_size[1] as f32);

        let cameras = [
            Camera2D {
                zoom: cam_zoom,
                target: cam_target,
                render_target: Some(display_buffers[0].clone()),
                ..Default::default()
            },
            Camera2D {
                zoom: cam_zoom,
                target: cam_target,
                render_target: Some(display_buffers[1].clone()),
                ..Default::default()
            },
        ];
        set_camera(&cameras[0]);
        Self {
            registers: [0; 32],
            pc: 0,
            steps_since_start: 0,
            sp: 0,
            stack,
            rom: Vec::new(),
            rom_size: 0,
            memory,
            display_buffers,
            current_buffer: 1,
            opcode: 0,
            sprites: HashMap::new(),
            display_size: [display_size[0] as usize, display_size[1] as usize],
            cameras,
        }
    }

    pub fn load_rom(&mut self, rom_path: String){
        let data = fs::read(rom_path).unwrap();
        if data.len() > 65536*65536 {
            panic!("ROM file is too large!");
        }
        self.rom = data;
        self.rom_size = self.rom.len();
    }

    #[allow(dead_code)]
    pub fn load_rom_raw(&mut self, rom: &[u8]) {
        self.rom = rom.to_vec();
        self.rom_size = self.rom.len();
    }

    #[allow(dead_code)]
    pub fn get8rom(&mut self) -> u8 {
        let out = self.rom[self.pc as usize];
        self.pc = self.pc.wrapping_add(1);
        out
    }

    #[allow(dead_code)]
    pub fn get16rom(&mut self) -> u16 {
        let out = ((self.rom[self.pc as usize] as u16) << 8) | self.rom[(self.pc+1) as usize] as u16;
        self.pc = self.pc.wrapping_add(2);
        out
    }

    #[allow(dead_code)]
    pub fn get32rom(&mut self) -> u32 {
        let out = ((self.rom[self.pc as usize] as u32) << 24) |
            ((self.rom[(self.pc+1) as usize] as u32) << 16) |
            ((self.rom[(self.pc+2) as usize] as u32) << 8) |
            self.rom[(self.pc+3) as usize] as u32;

        self.pc = self.pc.wrapping_add(4);
        out
    }

    #[allow(dead_code)]
    pub fn peek8rom(&mut self, addr: u32) -> u8 {
        self.rom[addr as usize]
    }

    #[allow(dead_code)]
    pub fn peek16rom(&mut self, addr: u32) -> u16 {
        ((self.rom[addr as usize] as u16) << 8) | self.rom[(addr+1) as usize] as u16
    }

    #[allow(dead_code)]
    pub fn peek32rom(&mut self, addr: u32) -> u32 {
        ((self.rom[addr as usize] as u32) << 24) | ((self.rom[(addr+1) as usize] as u32) << 16) |
        ((self.rom[addr as usize] as u32) << 8) | self.rom[(addr+1) as usize] as u32
    }

    #[allow(dead_code)]
    pub fn get8(&mut self, addr: u32) -> u8 {
        self.memory[addr as usize]
    }

    #[allow(dead_code)]
    pub fn get16(&mut self, addr: u32) -> u16 {
        ((self.memory[addr as usize] as u16) << 8) | self.memory[(addr+1) as usize] as u16
    }

    #[allow(dead_code)]
    pub fn get32(&mut self, addr: u32) -> u32 {
        ((self.memory[addr as usize] as u32) << 24) |
            ((self.memory[(addr+1) as usize] as u32) << 16) |
            ((self.memory[(addr+2) as usize] as u32) << 8) |
            self.memory[(addr+3) as usize] as u32
    }

    pub fn draw_buffer(&mut self){
        set_default_camera();
        draw_texture_ex(
            &self.display_buffers[self.current_buffer].texture,
            0.,
            0.,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );
        set_camera(&self.cameras[self.current_buffer]);
    }

    pub fn step(&mut self) -> bool {
        if self.pc >= self.rom_size as u32 {
            return false;
        }
        self.opcode = self.get8rom();
        // println!("{:X?}", self.opcode);
        match self.opcode {
            0x00 => {self.pc -= 1},
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
            0x16 => self.op_draw_sprite(),
            0x17 => self.op_jgt(),
            0xFF => return false,
            _ => {}
        }
        self.steps_since_start = self.steps_since_start.wrapping_add(1);
        true
    }

    /// load an immediate (val2) into register (val1)
    pub fn op_mov(&mut self) {
        let reg = self.get8rom();
        let val = self.get32rom();
        self.registers[reg as usize] = val;
    }

    /// store reg1 (val1) into memory at addr - addr+3 (val2)
    pub fn op_store_i(&mut self) {
        let reg = self.get8rom();
        let addr = self.get32rom();
        self.memory[addr as usize] = ((self.registers[reg as usize] >> 24) & 0xff) as u8;
        self.memory[(addr+1) as usize] = ((self.registers[reg as usize] >> 16) & 0xff) as u8;
        self.memory[addr as usize] = ((self.registers[reg as usize] >> 8) & 0xff) as u8;
        self.memory[(addr+1) as usize] = (self.registers[reg as usize] & 0xff) as u8;
    }

    /// load reg1 (val1) using memory at addr - addr+3 (val2)
    pub fn op_load_i(&mut self) {
        let reg = self.get8rom();
        let addr = self.get32rom();
        self.registers[reg as usize] = self.get32(addr);
    }

    /// store reg1 (val1) into memory at the address specified by reg2 (val2)
    pub fn op_store(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let addr = self.registers[reg2 as usize];
        self.memory[addr as usize] = ((self.registers[reg1 as usize] >> 24) & 0xff) as u8;
        self.memory[(addr+1) as usize] = ((self.registers[reg1 as usize] >> 16) & 0xff) as u8;
        self.memory[(addr+2) as usize] = ((self.registers[reg1 as usize] >> 8) & 0xff) as u8;
        self.memory[(addr+3) as usize] = (self.registers[reg1 as usize] & 0xff) as u8;
    }

    /// load reg1 (val1) using memory at the address specified by reg2 (val2)
    pub fn op_load(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let addr = self.registers[reg2 as usize];
        self.registers[reg1 as usize] = self.get32(addr);
    }

    /// set reg1 (val1) to reg1 + reg2 (val2)
    pub fn op_add(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_add(self.registers[reg2 as usize]);
    }

    /// set reg1 (val1) to reg1 + immediate (val2)
    pub fn op_add_i(&mut self) {
        let reg1 = self.get8rom();
        let val = self.get32rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_add(val);
    }

    /// set reg1 (val1) to reg1 - reg2 (val2)
    pub fn op_sub(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_sub(self.registers[reg2 as usize]);
    }

    /// set reg1 (val1) to reg1 - immediate (val2)
    pub fn op_sub_i(&mut self) {
        let reg1 = self.get8rom();
        let val = self.get32rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].wrapping_sub(val);
    }

    /// set reg1 (val1) to reg1 * reg2 (val2)
    pub fn op_mul(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].mul(self.registers[reg2 as usize]);
    }

    /// set reg1 (val1) to reg1 / reg2 (val2)
    pub fn op_div(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        self.registers[reg1 as usize] = self.registers[reg1 as usize].div(self.registers[reg2 as usize]);
    }

    /// unconditional jump to addr (val1)
    pub fn op_jmp(&mut self) {
        let addr = self.get32rom();
        self.pc = addr;
    }

    /// jump to addr (val3) if reg1 (val1) equals reg2 (val2)
    pub fn op_jeq(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let addr = self.get32rom();
        if self.registers[reg1 as usize] == self.registers[reg2 as usize] {
            self.pc = addr;
        }
    }

    /// jump to addr (val3) if reg1 (val1) does not equal reg2 (val2)
    pub fn op_jne(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let addr = self.get32rom();
        if self.registers[reg1 as usize] != self.registers[reg2 as usize] {
            self.pc = addr;
        }
    }

    /// jump to addr (val3) if reg1 (val1) is greater than reg2 (val2)
    pub fn op_jgt(&mut self) {
        let reg1 = self.get8rom();
        let reg2 = self.get8rom();
        let addr = self.get32rom();
        if self.registers[reg1 as usize] > self.registers[reg2 as usize] {
            self.pc = addr;
        }
    }

    /// store a null terminated string located in rom at addr (val1) into memory starting at rom addr (val2)
    pub fn op_store_str(&mut self){
        let mut addr1 = self.get32rom();
        let mut addr2 = self.get32rom();
        let mut c = self.rom[addr1 as usize];
        while c != 0x00 {
            self.memory[addr2 as usize] = c;
            addr1 = addr1.wrapping_add(1);
            addr2 = addr2.wrapping_add(1);
            c = self.rom[addr1 as usize];
        }
        self.memory[addr2 as usize] = c;
    }

    /// print a null terminated string from memory starting at addr (val1)
    pub fn op_print_str_mem(&mut self){
        let mut addr = self.get32rom();
        let mut c = self.get8(addr);
        while c != 0x00 {
            print!("{}", c as char);
            addr = addr.wrapping_add(1);
            c = self.get8(addr);
        }
        std::io::stdout().flush().unwrap();
    }

    /// print a null terminated string from rom starting at addr (val1)
    pub fn op_print_str_rom(&mut self){
        let mut addr = self.get32rom();
        let mut c = self.rom[addr as usize];
        while c != 0x00 {
            print!("{}", c as char);
            addr += 1;
            c = self.rom[addr as usize];
        }
        std::io::stdout().flush().unwrap();
    }

    /// push current pc to stack and jump to addr (val1)
    pub fn op_call(&mut self) {
        let addr = self.get32rom();
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
        if self.sp > self.stack.len() as u16 {
            println!("Stack overflow");
            exit(1);
        }
        self.pc = addr;
    }

    /// pop top of stack to pc
    pub fn op_return(&mut self) {
        self.sp -= 1;
        self.pc = self.stack[self.sp as usize];
    }

    /// swap the drawing buffer with the displayed buffer (double buffering)
    pub fn op_swap_buffers(&mut self){
        if self.current_buffer == 0 {
            self.current_buffer = 1;
        } else {
            self.current_buffer = 0;
        }
        set_camera(&self.cameras[self.current_buffer]);
    }

    /// draw a pixel at x: reg1 (val1), y: reg2 (val2), with a 32 bit ARGB color (val3)
    pub fn op_draw_pixel(&mut self) {
        let x_pos = self.registers[self.get8rom() as usize] as usize % self.display_size[0];
        let y_pos = self.registers[self.get8rom() as usize] as usize % self.display_size[1];
        let color = self.get32rom();
        draw_rectangle(x_pos as f32, y_pos as f32, 1f32, 1f32, Color::from_hex(color));
    }

    /// draw a sprite from a rom address (val3) at x: reg1 (val1), y: reg2 (val2). load the sprite if it hasn't been loaded yet
    pub fn op_draw_sprite(&mut self) {
        let x_pos = self.registers[self.get8rom() as usize] as usize % self.display_size[0];
        let y_pos = self.registers[self.get8rom() as usize] as usize % self.display_size[1];
        let addr = self.get32rom();
        if !self.sprites.contains_key(&addr)  {
            self.load_sprite(addr);
        }
        draw_texture(&self.sprites[&addr], x_pos as f32, y_pos as f32, WHITE);
    }

    pub fn load_sprite(&mut self, addr: u32) {
        let mut rom_addr = addr;
        let width = self.peek16rom(rom_addr) as usize;
        let height = self.peek16rom(rom_addr + 2) as usize;
        rom_addr += 4;
        let mut bytes = vec![0u8; width * height * 4];
        // println!("{}", height*width as usize * 4);
        for j in 0..height {
            for i in 0..width {
                bytes[j * width + i] = self.peek8rom(rom_addr);
                bytes[j * width + i + 1] = self.peek8rom(rom_addr+1);
                bytes[j * width + i + 2] = self.peek8rom(rom_addr+2);
                bytes[j * width + i + 3] = self.peek8rom(rom_addr+3);
                rom_addr += 4;
            }
        }
        let texture = Texture2D::from_rgba8(width as u16, height as u16, &bytes);
        self.sprites.insert(addr, texture);

    }
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
    let rom_path = args.rom.unwrap_or("res/out.p".to_string());
    let mut state = Box::new(ChipP::new());
    state.load_rom(rom_path);
    request_new_screen_size(state.display_size[0] as f32 * 2.0, state.display_size[1] as f32 * 2.0);
    let mut last_frame = Instant::now();
    loop {
        if is_key_down(KeyCode::Escape) {
            break;
        }
        state.step();
        // draw_rectangle(0.0, 0.0, 32.0, 32.0, RED);
        if last_frame.elapsed().as_secs_f32() > 1f32 / 60f32 {
            last_frame = Instant::now();
            next_frame().await;
            state.draw_buffer();
        }
    }
}
