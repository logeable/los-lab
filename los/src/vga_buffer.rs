use core::ptr::write_volatile;

pub const VGA_BUFFER_BASE_ADDR: usize = 0xb8000;

pub fn test() {
    let sc = ScreenCharacter::new(b'A', Attribute::new(Pallete::Red, Pallete::Black, false));

    unsafe {
        write_volatile(
            (VGA_BUFFER_BASE_ADDR + 16 * 80 * 2) as *mut ScreenCharacter,
            sc,
        )
    };
}

#[allow(dead_code)]
#[derive(Debug)]
#[repr(u8)]
enum Pallete {
    Black,
    Blue,
    Green,
    Cyan,
    Red,
    Magenta,
    Bron,
    LightGray,
    DarkGray,
    LightBlue,
    LightGreen,
    LightCyan,
    LightRed,
    Pink,
    Yellow,
    White,
}

#[repr(transparent)]
struct Attribute(u8);

impl Attribute {
    fn new(foreground: Pallete, background: Pallete, blink: bool) -> Self {
        let blink_attr = if blink { 1 << 7 } else { 0 };
        Attribute(blink_attr | (((background as u8) << 4) & 0b111) | ((foreground as u8) & 0b1111))
    }
}

#[repr(C)]
struct ScreenCharacter {
    character: u8,
    attribute: Attribute,
}

impl ScreenCharacter {
    fn new(character: u8, attribute: Attribute) -> Self {
        Self {
            character,
            attribute,
        }
    }
}
