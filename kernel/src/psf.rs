use std::slice::from_raw_parts;

pub struct Font {
    glyphs: &'static [u8],
    charsize: usize,
}

impl Font {
    pub fn new(start: usize, end: usize) -> Option<Self> {
        let header = unsafe { &*(start as *const Header) };
        if !header.check() {
            return None;
        }

        assert!(!header.has_unicode(), "unimplemented");

        let glyphs = header.glyphs();

        Some(Self {
            glyphs,
            charsize: header.charsize as usize,
            width: header.width,
            header: header.height,
        })
    }

    fn glyph(&self) {
        //let bytesperline = (self.width+7)/8;

        /*
            let mut glyph: *const u8 = unsafe {(self as *const Self as *const u8).offset(
                (self.headersize as isize) +
                (self.bytesperglyph as isize) *
                (if (c as u32) < self.numglyph { c as isize } else { 0 })
            )};
        */
    }
}

#[repr(C)]
struct Header {
    magic: [u8; 4], // magic bytes to identify PSF
    version: u32,   // zero
    offset: u32,    // offset of bitmaps in file, 32
    flags: u32,     // 0 if there's no unicode table
    len: u32,       // number of glyphs
    charsize: u32,  // size of each glyph
    height: u32,    // height in pixels
    width: u32,     // width in pixels
}

impl Header {
    fn check(&self) -> bool {
        self.magic[0] == 0x72 &&
        self.magic[0] == 0xB2 &&
        self.magic[0] == 0x4A &&
        self.magic[0] == 0x86 &&
        self.version == 0
    }

    fn has_unicode(&self) -> bool {
        self.flags & 0x01 != 0
    }

    fn glyphs(&self, start: usize, end: usize) -> &'static [u8] {
        let p = (start + self.offset) as *const u8;
        let len = (self.len * self.charsize) as usize;
        assert!(p as usize + len <= end);
        unsafe { from_raw_parts(p, len) }
    }

    fn unicode(&self, start: usize, end: usize) -> &'static [u8] {
        let p = (start + self.offset + self.len * self.charsize) as *const u8;
        let len = if self.has_unicode() {
            end - p
        } else {
            0
        };
        unsafe { from_raw_parts(p, len) }
    }

    /*
    fn psf_init() {
        let mut glyph: u16 = 0;
        // cast the address to PSF header struct
        PSF_font *font = (PSF_font*)&_binary_font_psf_start;
        // is there a unicode table?
        if(font->flags) {
            // get the offset of the table
            char *s = (char *)(
                (unsigned char*)&_binary_font_psf_start +
                font->headersize +
                font->numglyph * font->bytesperglyph
            );
            // allocate memory for translation table
            unicode = calloc(USHRT_MAX, 2);
            while s>_binary_font_psf_end {
                let uc: u16 = (uint16_t)((unsigned char *)s[0]);
                if uc == 0xFF {
                    glyph++;
                    s++;
                    continue;
                } else if(uc & 128) {
                    // UTF-8 to unicode
                    if (uc & 32) == 0 {
                        uc = ((s[0] & 0x1F)<<6)+(s[1] & 0x3F);
                        s++;
                    } else if (uc & 16) == 0 {
                        uc = ((((s[0] & 0xF)<<6)+(s[1] & 0x3F))<<6)+(s[2] & 0x3F);
                        s+=2;
                    } else if (uc & 8) == 0 {
                        uc = ((((((s[0] & 0x7)<<6)+(s[1] & 0x3F))<<6)+(s[2] & 0x3F))<<6)+(s[3] & 0x3F);
                        s+=3;
                    } else {
                        uc = 0;
                    }
                }
                // save translation
                unicode[uc] = glyph;
                s++;
            }
        } else {
            unicode = NULL;
        }
    }
    */
}
