use pi::mbox;
use std::ptr::write_volatile;
use std::mem::transmute;

pub struct FrameBuffer {
    buffer: *mut u8,
    width: u32,
    height: u32,
    pitch: u32,
}

impl FrameBuffer {
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn pitch(&self) -> u32 { self.pitch }

    pub unsafe fn offset_bytes(&mut self, offset: isize) -> *mut u8 {
        self.buffer.offset(offset)
    }

    pub fn fill_rgba(&self, rgba: u32) {
        let mut p = self.buffer;
        let stride = self.pitch as usize - self.width as usize * 4;
        for _ in 0..self.height {
            for _ in 0..self.width {
                unsafe {
                    write_volatile(p as *mut u32, rgba);
                    p = p.add(4);
                }
            }
            unsafe { p = p.add(stride); }
        }
    }
}

pub struct FrameBufferInfo {
    pub width: u32,
    pub height: u32,
    pub virtual_width: u32,
    pub virtual_height: u32,
    pub x_offset: u32,
    pub y_offset: u32,
    pub depth: u32,
    pub rgb: bool,
}

pub fn init(info: FrameBufferInfo) -> Option<FrameBuffer> {
    mbox::Mailbox::new().tag_message(&[
        mbox::Tag::SET_PHYSICAL_WIDTH_HEIGHT as u32, 8, 8, info.width, info.height,
        mbox::Tag::SET_VIRTUAL_WIDTH_HEIGHT as u32, 8, 8, info.virtual_width, info.virtual_height,
        mbox::Tag::SET_VIRTUAL_OFFSET as u32, 8, 8, info.x_offset, info.y_offset,
        mbox::Tag::SET_COLOUR_DEPTH as u32, 4, 4, info.depth,
        mbox::Tag::SET_PIXEL_ORDER as u32, 4, 4, info.rgb as u32, //RGB, not BGR preferably
        mbox::Tag::ALLOCATE_FRAMEBUFFER as u32, 8, 8, 4096, 0, // pointer/size // 8, 4, 16, 0
        mbox::Tag::GET_PITCH as u32, 4, 4, 0,
    ]).and_then(|buf| {
        if buf[20] == 32 && buf[28] != 0 {
            let addr = mbox::gpu2arm(buf[28] as usize) as *mut u8;
            Some(FrameBuffer {
                width: buf[5],
                height: buf[6],
                pitch: buf[33],
                buffer: addr,
            })
        } else {
            None
        }
    })
}

/// PC Screen Font as used by Linux Console
#[repr(packed)]
pub struct Font {
    _magic: u32,
    _version: u32,
    headersize: u32,
    _flags: u32,
    numglyph: u32,
    bytesperglyph: u32,
    height: u32,
    width: u32,
}

static FONT: &[u8] = include_bytes!("../font.psf");

pub fn font() -> &'static Font {
    unsafe {
        transmute(FONT.as_ptr())
    }
}

impl Font {
    pub fn uprint(&self, fb: &mut FrameBuffer, mut x: isize, mut y: isize, s: &str, fg: u32, bg: u32) {
        let bytesperline = (self.width+7)/8;

        // draw next character if it's not zero
        for c in s.chars() {
            // get the offset of the glyph.
            // Need to adjust this to support unicode table
            let mut glyph: *const u8 = unsafe {(self as *const Self as *const u8).offset(
                (self.headersize as isize) +
                (self.bytesperglyph as isize) *
                (if (c as u32) < self.numglyph { c as isize } else { 0 })
            )};

            // calculate the offset on screen
            let width = self.width as isize;
            let height = self.height as isize;
            let pitch = fb.pitch() as isize;
            let mut offs = y * height * pitch + x * width * 4;

            // handle carrige return
            if c == '\r' {
                x = 0;
            } else if c == '\n' { // new line
                x = 0;
                y += 1;
            } else {
                // display a character
                for _ in 0..self.height {
                    // display one row
                    let mut line = offs;
                    let mut mask = 1 << (self.width-1);
                    for _ in 0..self.width {
                        // if bit set, we use white color, otherwise black
                        unsafe {
                        *(fb.offset_bytes(line) as *mut u32) =
                            if *(glyph as *const u32) & mask != 0 { fg } else { bg };
                        }
                        mask >>= 1;
                        line += 4;
                    }
                    // adjust to next line
                    unsafe { glyph = glyph.offset(bytesperline as isize) };
                    offs += pitch;
                }

                x += 1;
            }
        }
    }
}
