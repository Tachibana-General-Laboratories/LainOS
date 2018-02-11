use mbox;

pub struct Lfb {
    width: u32,
    height: u32,
    pitch: u32,
    lfb: *mut u8,
}

impl Lfb {
    pub fn fill_rgba(&self, rgba: u32) {
        let mut p = self.lfb;
        let stride = self.pitch as isize - self.width as isize * 4;
        for _ in 0..self.height {
            for _ in 0..self.width {
                unsafe {
                    *(p as *mut u32) = rgba;
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
