use pi::mbox;
use std::mem::transmute;
use std::slice::from_raw_parts;

struct Info {
    width: u32,         // width of the display
    height: u32,        // height of the display
    virtual_width: u32, // width of the virtual framebuffer
    virtual_height: u32,// height of the virtual framebuffer
    pitch: u32,         // number of bytes per row
    depth: u32,         // number of bits per pixel
    x_offset: u32,      // x of the upper left corner of the virtual fb
    y_offset: u32,      // y of the upper left corner of the virtual fb
    framebuffer: u32,   // pointer to the start of the framebuffer
    size: u32,          // number of bytes in the framebuffer
}

#[derive(Debug)]
pub struct FrameBuffer {
    buffer: *mut u8,
    width: u32,
    height: u32,
    pitch: u32,

    x_offset: u32,
    y_offset: u32,

    next: bool,
}

impl FrameBuffer {
    pub fn new(width: u32, height: u32, depth: u32) -> Option<Self> {
        let info = FrameBufferInfo {
            width,
            height,
            virtual_width: width,
            virtual_height: height,
            x_offset: 0,
            y_offset: 0,
            depth,
            rgb: false,
        };

        init(info)
    }

    pub fn flush(&mut self) -> bool {
        let y = if self.next {
            self.height
        } else {
            0
        };
        let ok = mbox::Mailbox::new().tag_message(&[
            mbox::Tag::SET_VIRTUAL_OFFSET as u32, 8, 8, self.x_offset, y,
        ]).is_some();

        if ok {
            self.y_offset = y;
        }

        ok
    }

    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn pitch(&self) -> u32 { self.pitch }

    pub unsafe fn set(&mut self, offset: isize, color: u32) {
        (self.buffer.offset(offset) as *mut u32).write_volatile(color);
    }

    pub fn fill_rgba(&self, rgba: u32) {
        let mut p = self.buffer;
        let stride = self.pitch as usize - self.width as usize * 4;
        for _ in 0..self.height {
            for _ in 0..self.width {
                unsafe {
                    (p as *mut u32).write_volatile(rgba);
                    p = p.add(4);
                }
            }
            unsafe { p = p.add(stride); }
        }
    }
}

struct FrameBufferInfo {
    width: u32,
    height: u32,
    virtual_width: u32,
    virtual_height: u32,
    x_offset: u32,
    y_offset: u32,
    depth: u32,
    rgb: bool,
}

fn init(info: FrameBufferInfo) -> Option<FrameBuffer> {
    mbox::Mailbox::new().tag_message(&[
        mbox::Tag::SET_PHYSICAL_WIDTH_HEIGHT as u32, 8, 8, info.width, info.height,
        mbox::Tag::SET_VIRTUAL_WIDTH_HEIGHT as u32, 8, 8, info.virtual_width, info.virtual_height,
        mbox::Tag::SET_VIRTUAL_OFFSET as u32, 8, 8, info.x_offset, info.y_offset,
        mbox::Tag::SET_COLOUR_DEPTH as u32, 4, 4, info.depth,
        mbox::Tag::SET_PIXEL_ORDER as u32, 4, 4, info.rgb as u32, //RGB, not BGR preferably
        //mbox::Tag::ALLOCATE_FRAMEBUFFER as u32, 8, 8, 4096, 0, // pointer/size // 8, 4, 16, 0
        //mbox::Tag::ALLOCATE_FRAMEBUFFER as u32, 8, 4, 16, 0, // pointer/size // 8, 4, 16, 0
        mbox::Tag::ALLOCATE_FRAMEBUFFER as u32, 8, 8, 16, 0, // pointer/size // 8, 4, 16, 0
        mbox::Tag::GET_PITCH as u32, 4, 4, 0,
    ]).and_then(|buf| {
        if buf[20] == 32 && buf[28] != 0 {
            let addr = mbox::gpu2arm(buf[28] as usize) as *mut u8;
            Some(FrameBuffer {
                width: buf[5],
                height: buf[6],
                pitch: buf[33],
                buffer: addr,
                x_offset: info.x_offset,
                y_offset: info.y_offset,

                next: false,
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

static FONT: &[u8] = include_bytes!("../ext/font.psf");

pub fn font() -> &'static Font {
    unsafe {
        transmute(FONT.as_ptr())
    }
}

impl Font {
    // TODO: Need to adjust this to support unicode table
    fn glyph(&self, c: char) -> &[u8] {
        let c = c as u32;

        let offset = if c < self.numglyph {
            self.headersize + self.bytesperglyph * c
        } else {
            self.headersize
        };

        let addr = (self as *const Self as *const u8);
        unsafe {
            from_raw_parts(addr.offset(offset as isize), self.bytesperglyph as usize)
        }
    }
    pub fn uprint(&self, fb: &mut FrameBuffer, mut x: isize, mut y: isize, s: &str, fg: u32, bg: u32) {
        let bytesperline = (self.width+7)/8;

        assert!(bytesperline > 0 && bytesperline <= 4);

        // calculate the offset on screen
        let width = self.width as isize;
        let height = self.height as isize;
        let pitch = fb.pitch() as isize;

        // draw next character if it's not zero
        for c in s.chars() {
            let glyph = self.glyph(c);

            let mut offs = y * height * pitch + x * width * 4;

            // handle carrige return
            if c == '\r' {
                x = 0;
            } else if c == '\n' { // new line
                x = 0;
                y += 1;
            } else {
                for line in glyph.windows(bytesperline as usize) {
                    let mut c = 0;

                    for i in 0..bytesperline {
                        c |= (line[i as usize] as u32) << (i as u32) * 8;
                    }

                    // display one row
                    let mut start = offs;
                    offs += pitch;

                    let mut mask = 1 << (self.width-1);
                    for _ in 0..width {
                        // if bit set, we use white color, otherwise black
                        let is = c & mask != 0;
                        unsafe {
                            fb.set(start, if is {
                                fg
                            } else {
                                bg
                            });
                        }
                        mask >>= 1;
                        start += 4;
                    }
                }
                x += 1;
            }
        }
    }
}
