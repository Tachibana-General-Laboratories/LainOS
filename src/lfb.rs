use mbox;
use core::ptr;

pub struct Lfb {
    lfb: *mut u8,
    width: u32,
    height: u32,
    pitch: u32,
}

impl Lfb {
    pub fn fill_rgba(&self, rgba: u32) {
        let mut p = self.lfb;
        let stride = self.pitch as isize - self.width as isize * 4;
        for _ in 0..self.height {
            for _ in 0..self.width {
                unsafe {
                    ptr::write(p as *mut u32, rgba);
                    p = p.offset(4);
                }
            }
            unsafe { p = p.offset(stride); }
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

pub fn init(info: FrameBufferInfo) -> Option<Lfb> {
    let b = unsafe { &mut mbox::BUFFER };

    b[ 0].write(35*4);
    b[ 1].write(mbox::REQUEST);
    //set phy wh
    b[ 2].write(0x48003);
    b[ 3].write(8);
    b[ 4].write(8);
    b[ 5].write(info.width);
    b[ 6].write(info.height);

    //set virt wh
    b[ 7].write(0x48004);
    b[ 8].write(8);
    b[ 9].write(8);
    b[10].write(info.virtual_width);
    b[11].write(info.virtual_height);

    //set virt offset
    b[12].write(0x48009);
    b[13].write(8);
    b[14].write(8);
    b[15].write(info.x_offset);
    b[16].write(info.y_offset);

    //set depth
    b[17].write(0x48005);
    b[18].write(4);
    b[19].write(4);
    b[20].write(info.depth);          //FrameBufferInfo.depth

    b[21].write(0x48006); //set pixel order
    b[22].write(4);
    b[23].write(4);
    b[24].write(info.rgb as u32);           //RGB, not BGR preferably

    b[25].write(0x40001); //get framebuffer, gets alignment on request
    b[26].write(8);
    b[27].write(8);
    b[28].write(4096);        //FrameBufferInfo.pointer
    b[29].write(0);           //FrameBufferInfo.size

    b[30].write(0x40008); //get pitch
    b[31].write(4);
    b[32].write(4);
    b[33].write(0);           //FrameBufferInfo.pitch

    b[34].write(mbox::TAG_LAST);

    let val = unsafe { mbox::call(mbox::Channel::PROP1) };

    if val && b[20].read() == 32 && b[28].read() != 0 {
        let w = b[28].read() & 0x3FFFFFFF;
        b[28].write(w);

        Some(Lfb {
            width: b[5].read(),
            height: b[6].read(),
            pitch: b[33].read(),
            lfb: b[28].read() as *mut u8,
        })
    } else {
        None
    }
}

/// PC Screen Font as used by Linux Console
#[repr(packed)]
pub struct Font {
    magic: u32,
    version: u32,
    headersize: u32,
    flags: u32,
    numglyph: u32,
    bytesperglyph: u32,
    height: u32,
    width: u32,
}

pub static FONT: &[u8] = include_bytes!("../font.psf");

impl Font {
    pub unsafe fn uprint(&self, fb: Lfb, mut x: isize, mut y: isize, s: &str, fg: u32, bg: u32) {
        let bytesperline = (self.width+7)/8;

        // draw next character if it's not zero
        for c in s.chars() {
            // get the offset of the glyph. Need to adjust this to support unicode table
            let mut glyph: *const u8 = (self as *const Self as *const u8).offset(
                (self.headersize as isize) +
                (self.bytesperglyph as isize) *
                (if (c as u32) < self.numglyph { c as isize } else { 0 })
            );

            // calculate the offset on screen
            let width = self.width as isize;
            let height = self.height as isize;
            let pitch = fb.pitch as isize;
            let mut offs = y * height * pitch + x * (width) * 4;

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
                        *(fb.lfb.offset(line) as *mut u32) =
                            if ((*glyph) as u32) & mask != 0 { fg } else { bg };
                        mask >>= 1;
                        line += 4;
                    }
                    // adjust to next line
                    glyph = glyph.offset(bytesperline as isize);
                    offs += pitch;
                }

                x += 1;
            }
        }
    }
}
