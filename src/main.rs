mod font;
mod keyboard;

use bitvec::prelude::*;
use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::surface::Surface;
use std::cell::RefCell;
use std::cmp::min;
use std::collections::HashSet;
use std::fs::read;
use std::time::{Duration, Instant};

use crate::font::FONT;
use crate::keyboard::hex_keypad;

const _PIXEL_OFF_COLOR: Color = Color::BLACK;
const _PIXEL_ON_COLOR: Color = Color::WHITE;
const DISPLAY_WIDTH: u32 = 64;
const DISPLAY_HEIGHT: u32 = 32;
const DISPLAY_SCALE: u32 = 10;
const EXEC_SPEED: u32 = 700;

#[derive(PartialEq, Debug)]
struct Emulator {
    display_buffer: RefCell<Vec<u8>>,
    display_flag: bool,
    memory: Vec<u8>,
    stack: Vec<u16>,
    program_counter: RefCell<usize>,
    index: u16,
    registers: [u8; 16],
    super_chip: bool,
    delay_timer: RefCell<u8>,
    sound_timer: RefCell<u8>,
    keys_down: HashSet<usize>,
}

impl Emulator {
    fn new(super_chip: bool) -> Self {
        let mut emu = Emulator {
            display_buffer: RefCell::new(vec![0; (DISPLAY_HEIGHT * DISPLAY_WIDTH / 8) as usize]),
            display_flag: false,
            memory: vec![0; 4096],
            stack: Vec::new(),
            index: 0,
            program_counter: RefCell::new(0x200),
            registers: [0; 16],
            keys_down: HashSet::new(),
            delay_timer: RefCell::new(0),
            sound_timer: RefCell::new(0),
            super_chip,
        };
        emu.memory.splice(0x50..0x9F, FONT);
        emu
    }

    fn decrement_timers(&self) {
        if *self.delay_timer.borrow() > 0 {
            *self.delay_timer.borrow_mut() -= 1;
        }
        if *self.sound_timer.borrow() > 0 {
            *self.sound_timer.borrow_mut() -= 1;
        }
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
        let (r, l) = instruction.split_at(8);
        let nibbles: Vec<u8> = instruction.chunks(4).map(|x| x.load::<u8>()).collect();
        let ((_a, b), (c, d)) = (r.split_at(4), l.split_at(4));
        let x: usize = b.load_be();
        let y: usize = c.load_be();
        let vx = self.registers[x];
        let vy = self.registers[y];
        let nn = instruction[8..].load_be::<u8>();
        let nnn = instruction[4..].load_be::<u16>();
        // let u = (a & 0b11110000) >> 4;
        match nibbles[0] {
            0x0 => match instruction[4..].load_be::<u16>() {
                0xEE => {
                    *self.program_counter.borrow_mut() =
                        usize::from(self.stack.pop().expect("tried to pop from an empty stack"))
                }
                0xE0 => self.clear_screen(),
                _ => {}
            },
            0x1 => self.jump(nnn),
            0x2 => self.jump_subroutine(nnn),
            0x3 => {
                if vx == nn {
                    self.skip()
                }
            }
            0x4 => {
                if vx != nn {
                    self.skip()
                }
            }
            0x5 => {
                if vx == vy {
                    self.skip()
                }
            }
            0x6 => {
                self.registers[x] = nn;
            }
            0x7 => {
                self.registers[x] = vx.wrapping_add(nn);
            }
            0x8 => {
                self.registers[x] = match d.load::<usize>() {
                    0 => vy,
                    1 => vx | vy,
                    2 => vx & vy,
                    3 => vx ^ vy,
                    4 => {
                        if vx.checked_add(vy) == None {
                            self.registers[0xF] = 1
                        };
                        vx.wrapping_add(vy)
                    }
                    5 => {
                        self.registers[0xF] = 1;
                        if vx < vy {
                            self.registers[0xF] = 0;
                        };
                        vx.wrapping_sub(vy)
                    }
                    6 => {
                        self.registers[0xF] = if vx.view_bits::<Msb0>()[7] { 1 } else { 0 };
                        vx >> 1
                    }
                    7 => {
                        self.registers[0xF] = 1;
                        if vy < vx {
                            self.registers[0xF] = 0
                        };
                        vy.wrapping_sub(vx)
                    }
                    0xe => {
                        self.registers[0xF] = if vx.view_bits::<Msb0>()[0] { 1 } else { 0 };
                        vx << 1
                    }
                    _ => {
                        return Err(format!(
                            "invalid opcode {:x?}",
                            instruction.load_be::<u16>()
                        ))
                    }
                }
            }
            0x9 => {
                if vx != vy {
                    self.skip()
                }
            }
            0xA => self.index = nnn,
            0xB => {
                if self.super_chip {
                    self.jump(u16::from(nn + vx))
                } else {
                    self.jump(nnn + u16::from(self.registers[0]))
                }
            }
            0xC => self.registers[x] = random::<u8>() & nn,
            0xD => {
                self.display_flag = true;
                let x_coord: u8 = vx % 64;
                let y_coord: u8 = vy % 32;
                self.registers[0xF] = 0;
                let n: u8 = d.load_be();
                for j in y_coord..min(y_coord + n, u8::try_from(DISPLAY_HEIGHT).unwrap()) {
                    let m = usize::from(self.index) + usize::from(j - y_coord);
                    let row = self.memory[m..m + 1].view_bits::<Msb0>();
                    // println!("row {:x?}", row);
                    for i in x_coord..min(x_coord + 8, u8::try_from(DISPLAY_WIDTH).unwrap()) {
                        if self.get_pixel(i, j) && row[usize::from(i - x_coord)] {
                            self.set_pixel(i, j, false);
                            self.registers[0xF] = 1;
                        } else if !self.get_pixel(i, j) && row[usize::from(i - x_coord)] {
                            self.set_pixel(i, j, true);
                        }
                    }
                }
                // println!("display buffer dump: {:x?}", &self.display_buffer.borrow())
            }
            0xE => match nn {
                0x9E => {
                    if self.keys_down.contains(&usize::from(vx)) {
                        self.skip()
                    }
                }
                0xA1 => {
                    if !self.keys_down.contains(&usize::from(vx)) {
                        self.skip()
                    }
                }
                _ => {
                    return Err(format!(
                        "invalid opcode {:x?}",
                        instruction.load_be::<u16>()
                    ))
                }
            },
            0xF => match nn {
                0x07 => {
                    self.registers[x] = *self.delay_timer.borrow();
                }
                0x0A => {
                    if self.keys_down.is_empty() {
                        *self.program_counter.borrow_mut() -= 2;
                    } else {
                        self.registers[x] =
                            u8::try_from(*self.keys_down.iter().nth(0).unwrap()).unwrap();
                    }
                }
                0x15 => {
                    *self.delay_timer.borrow_mut() = vx;
                }
                0x18 => {
                    *self.sound_timer.borrow_mut() = vx;
                }
                0x1E => {
                    self.index += u16::from(vx);
                }
                0x29 => {
                    self.index = 0x50 + u16::from(vx * 5);
                }
                0x33 => {
                    let hundreds = vx / 100;
                    let tens = (vx % 100) / 10;
                    let ones = vx % 10;
                    self.memory[usize::from(self.index)] = hundreds;
                    self.memory[usize::from(self.index) + 1] = tens;
                    self.memory[usize::from(self.index) + 2] = ones;
                }
                0x55 => {
                    for i in 0..x + 1 {
                        self.memory[usize::from(self.index) + i] = self.registers[i];
                    }
                }
                0x65 => {
                    for i in 0..x + 1 {
                        self.registers[i] = self.memory[usize::from(self.index) + i];
                    }
                }
                _ => {
                    return Err(format!(
                        "invalid opcode {:x?}",
                        instruction.load_be::<u16>()
                    ))
                }
            },
            _ => {
                return Err(format!(
                    "invalid opcode {:x?}",
                    instruction.load_be::<u16>()
                ))
            }
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

    fn skip(&mut self) {
        *self.program_counter.borrow_mut() += 2;
    }
}

pub fn main() -> Result<(), String> {
    let mut emulator = Emulator::new(false);
    let path = std::env::args().nth(1).expect("no file given");
    let prog: Vec<u8> = read(path).map_err(|e| e.to_string())?;

    let mut now = Instant::now();

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
                    Event::KeyDown {
                        scancode: Some(scancode),
                        ..
                    } => {
                        if let Some(val) = hex_keypad(scancode) {
                            emulator.keys_down.insert(val);
                        }
                    }
                    Event::KeyUp {
                        scancode: Some(scancode),
                        ..
                    } => {
                        if let Some(val) = hex_keypad(scancode) {
                            emulator.keys_down.remove(&val);
                        }
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

        if emulator.display_flag {
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
        }
        emulator.display_flag = false;

        if now.elapsed() > Duration::from_micros(1000000 / 60) {
            emulator.decrement_timers();
            now = Instant::now();
        }
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / EXEC_SPEED));
    }
    Ok(())
}
