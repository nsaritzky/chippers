use sdl2::keyboard::Scancode;

pub fn hex_keypad(scancode: Scancode) -> Option<usize> {
    match scancode {
        Scancode::X => Some(0),
        Scancode::Num1 => Some(1),
        Scancode::Num2 => Some(2),
        Scancode::Num3 => Some(3),
        Scancode::Q => Some(4),
        Scancode::W => Some(5),
        Scancode::E => Some(6),
        Scancode::A => Some(7),
        Scancode::S => Some(8),
        Scancode::D => Some(9),
        Scancode::Z => Some(0xA),
        Scancode::C => Some(0xB),
        Scancode::Num4 => Some(0xC),
        Scancode::R => Some(0xD),
        Scancode::F => Some(0xE),
        Scancode::V => Some(0xF),
        _ => None,
    }
}
