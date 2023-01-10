mod font;

use bitvec::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use std::cell::RefCell;
use std::cmp::min;
use std::fs::read;
use std::time::Duration;

use crate::font::FONT;

const _PIXEL_OFF_COLOR: Color = Color::BLACK;
const _PIXEL_ON_COLOR: Color = Color::WHITE;
const DISPLAY_WIDTH: u32 = 64;
const DISPLAY_HEIGHT: u32 = 32;
const DISPLAY_SCALE: u32 = 10;
const EXEC_SPEED: u32 = 700;

#[derive(PartialEq, Debug)]
struct Emulator {
    display_buffer: RefCell<Vec<u8>>,
    memory: Vec<u8>,
    stack: Vec<u16>,
    program_counter: RefCell<usize>,
    index: u16,
    registers: [u8; 16],
    super_chip: bool,
}

impl Emulator {
    fn new(super_chip: bool) -> Self {
        let mut emu = Emulator {
            display_buffer: RefCell::new(vec![0; (DISPLAY_HEIGHT * DISPLAY_WIDTH / 8) as usize]),
            memory: vec![0; 4096],
            stack: Vec::new(),
            index: 0,
            program_counter: RefCell::new(0x200),
            registers: [0; 16],
            super_chip,
        };
        emu.memory.splice(0x50..0x9F, FONT);
        emu
    }

    fn get_pixel(&self, x: u8, y: u8) -> bool {
        let buf = self.display_buffer.borrow();
        let display_bits = buf.view_bits::<Msb0>();
        display_bits[usize::from(y) * usize::try_from(DISPLAY_WIDTH).unwrap() + usize::from(x)]
    }

    fn set_pixel(&self, x: u8, y: u8, state: bool) {
        let mut buf = self.display_buffer.borrow_mut();
        let display_bits = buf.view_bits_mut::<Msb0>();
        let n = usize::from(y) * usize::try_from(DISPLAY_WIDTH).unwrap() + usize::from(x);
        display_bits.set(n, state)
    }

    fn fetch(&self) -> BitVec<u8, Msb0> {
        let pc = *self.program_counter.borrow();
        let result = &self.memory[pc..pc + 2];
        self.program_counter.replace(pc + 2);
        // println!("fetched: {:x?}", result);
        BitVec::<u8, _>::from_slice(result)
    }

    fn decode(&mut self, instruction: BitVec<u8, Msb0>) -> Result<(), String> {
        let (x, y) = instruction.split_at(8);
        let nibbles: Vec<u8> = instruction.chunks(4).map(|x| x.load::<u8>()).collect();
        let ((_a, b), (c, d)) = (x.split_at(4), y.split_at(4));
        // let u = (a & 0b11110000) >> 4;
        match nibbles[0] {
            0x0 => match b.load::<u8>() {
                0xEE => {
                    *self.program_counter.borrow_mut() =
                        self.stack.pop().expect("tried to pop from an empty stack") as usize
                }
                0xE0 => self.clear_screen(),
                _ => {}
            },
            0x1 => {
                let addr = instruction[4..].load_be::<u16>();
                self.jump(addr)
            }
            0x2 => {
                let addr = instruction[4..].load_be::<u16>();
                self.jump_subroutine(addr)
            }
            0x3 => {
                let n: u8 = instruction[8..].load_be();
                if self.registers[b.load::<usize>()] == n {
                    *self.program_counter.borrow_mut() += 2;
                }
            }
            0x4 => {
                let n: u8 = instruction[8..].load_be();
                if self.registers[b.load::<usize>()] != n {
                    *self.program_counter.borrow_mut() += 2;
                }
            }
            0x5 => {
                if self.registers[b.load::<usize>()] == self.registers[c.load::<usize>()] {
                    *self.program_counter.borrow_mut() += 2;
                }
            }
            0x6 => {
                let n: u8 = instruction[8..].load_be();
                self.set_x(b.load_be(), n);
            }
            0x7 => {
                let n: u8 = instruction[8..].load_be();
                self.add_x(b.load_be(), n)
            }
            0x8 => {
                let x = self.registers[b.load::<usize>()];
                let y = self.registers[c.load::<usize>()];
                self.registers[b.load::<usize>()] = match d.load::<usize>() {
                    0 => y,
                    1 => x | y,
                    2 => x & y,
                    3 => x ^ y,
                    4 => {
                        if x.checked_add(y) == None {
                            self.registers[0xF] = 1
                        };
                        x.wrapping_add(y)
                    }
                    5 => {
                        self.registers[0xF] = 1;
                        if x > y {
                            self.registers[0xF] = 0;
                        };
                        x.wrapping_sub(y)
                    }
                    7 => {
                        self.registers[0xF] = 1;
                        if y > x {
                            self.registers[0xF] = 0
                        };
                        y.wrapping_sub(x)
                    }
                    _ => return Err(format!("invalid opcode {:x?}", instruction)),
                }
            }
            0x9 => {
                if self.registers[b.load::<usize>()] != self.registers[c.load::<usize>()] {
                    *self.program_counter.borrow_mut() += 2;
                }
            }
            0xA => {
                let addr = instruction[4..].load_be::<u16>();
                self.index = addr;
            }
            0xD => {
                let x: u8 = self.registers[b.load::<usize>()] % 64;
                let y: u8 = self.registers[c.load::<usize>()] % 32;
                self.registers[0xF] = 0;
                let n: u8 = d.load();
                for j in y..min(y + n, u8::try_from(DISPLAY_HEIGHT).unwrap() - 1) {
                    let m = usize::from(self.index) + usize::from(j - y);
                    let row = self.memory[m..m + 1].view_bits::<Msb0>();
                    // println!("row {:x?}", row);
                    for i in x..min(x + 8, u8::try_from(DISPLAY_WIDTH).unwrap() - 1) {
                        if self.get_pixel(i, j) && row[usize::from(i - x)] {
                            self.set_pixel(i, j, false);
                            self.registers[0xF] = 1;
                        } else if !self.get_pixel(i, j) && row[usize::from(i - x)] {
                            self.set_pixel(i, j, true);
                        }
                    }
                }
                // println!("display buffer dump: {:x?}", &self.display_buffer.borrow())
            }
            _ => return Err(format!("invalid opcode {:x?}", instruction)),
        }
        Ok(())
    }

    fn clear_screen(&mut self) {
        self.display_buffer.borrow_mut().fill(0);
    }

    fn jump(&mut self, n: u16) {
        // println!("Jumping to {:x?}", n);
        self.program_counter.replace(n as usize);
    }

    fn jump_subroutine(&mut self, n: u16) {
        self.stack
            .push(u16::try_from(*self.program_counter.borrow()).unwrap());
        *self.program_counter.borrow_mut() = usize::from(n);
    }

    fn set_x(&mut self, x: u8, n: u8) {
        self.registers[usize::from(x)] = n;
    }

    fn add_x(&mut self, x: u8, n: u8) {
        let m = self.registers[usize::from(x)];
        self.registers[usize::from(x)] = n.wrapping_add(m);
    }
}

pub fn main() -> Result<(), String> {
    let mut emulator = Emulator::new(false);
    let path = std::env::args().nth(1).expect("no file given");
    let prog: Vec<u8> = read(path).map_err(|e| e.to_string())?;

    emulator.memory.splice(0x200..0x200 + prog.len(), prog);

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window(
            "rust-sdl2 demo",
            DISPLAY_WIDTH * DISPLAY_SCALE,
            DISPLAY_HEIGHT * DISPLAY_SCALE,
        )
        .position_centered()
        .build()
        .expect("could not initialize video subsystem");

    // let mut canvas = window
    //     .into_canvas()
    //     .build()
    //     .expect("could not make a canvas");

    let mut event_pump = sdl_context.event_pump()?;

    let window_surface = window.surface(&event_pump)?;
    let pixel_format = window_surface.pixel_format();
    window_surface.finish()?;

    'running: loop {
        // canvas.set_draw_color(PIXEL_OFF_COLOR);
        // canvas.clear();
        // i = (i + 1) % 600;
        // canvas.set_draw_color(PIXEL_ON_COLOR);
        // canvas.draw_point(Point::new(i, i))?;
        {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => {
                        break 'running;
                    }
                    // Event::KeyDown {
                    //     scancode: Some(Scancode::S),
                    //     ..
                    // } => surface.with_lock_mut(|buf| buf.fill(1)),
                    _ => {}
                }
            }
        }
        let instruction = emulator.fetch();
        emulator.decode(instruction)?;
        let mut window_surface = window.surface(&event_pump)?;
        // window_surface.set_palette(&palette)?;

        let mut expanded_buf = emulator.display_buffer.borrow_mut();

        let surface = Surface::from_data(
            &mut expanded_buf,
            DISPLAY_WIDTH.into(),
            DISPLAY_HEIGHT.into(),
            DISPLAY_WIDTH / 8,
            sdl2::pixels::PixelFormatEnum::Index1MSB,
        )?;

        // The rest of the game loop goes here...

        let converted = surface.convert(&pixel_format)?;
        converted.blit_scaled(None, &mut window_surface, None)?;
        window_surface.finish()?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / EXEC_SPEED));
    }
    Ok(())
}
