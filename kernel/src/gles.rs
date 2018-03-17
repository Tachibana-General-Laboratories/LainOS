use pi::IO_BASE;
use pi::mbox::{self, gpu2arm};

use volatile::prelude::*;
use volatile::Volatile;

// REFERENCES
// https://docs.broadcom.com/docs/12358545
// http://latchup.blogspot.com.au/2016/02/life-of-triangle.html
// https://cgit.freedesktop.org/mesa/mesa/tree/src/gallium/drivers/vc4/vc4_draw.c

/*
#include <stdbool.h>                            // Needed for bool and true/false
#include <stdint.h>                                // Needed for uint8_t, uint32_t, etc
#include "rpi-smartstart.h"                        // Need for mailbox
#include "rpi-GLES.h"
*/


const v3d: *mut Volatile<u32> = (IO_BASE + 0xc00000) as *mut Volatile<u32>;

struct PerformanceCounter {
    Count: Volatile<u32>,
    Mapping: Volatile<u32>,
}

struct Counters {
    counters: [PerformanceCounter; 15],
}

// Registers shamelessly copied from Eric AnHolt

// Defines for v3d register offsets

const V3D_IDENT0: usize  = 0x000>>2; // V3D Identification 0 (V3D block identity)
const V3D_IDENT1: usize  = 0x004>>2; // V3D Identification 1 (V3D Configuration A)
const V3D_IDENT2: usize  = 0x008>>2; // V3D Identification 1 (V3D Configuration B)

//[Reserved; 1]

const V3D_SCRATCH: usize = 0x010>>2; // Scratch Register
//[Reserved; 3]
const V3D_L2CACTL: usize = 0x020>>2; // 2 Cache Control
const V3D_SLCACTL: usize = 0x024>>2; // Slices Cache Control
//[Reserved; 2]
const V3D_INTCTL: usize  = 0x030>>2; // Interrupt Control
const V3D_INTENA: usize  = 0x034>>2; // Interrupt Enables
const V3D_INTDIS: usize  = 0x038>>2; // Interrupt Disables
//[Reserved; 1]
const V3D_CT0CS: usize   = 0x100>>2; // Control List Executor Thread 0 Control and Status.
const V3D_CT1CS: usize   = 0x104>>2; // Control List Executor Thread 1 Control and Status.
const V3D_CT0EA: usize   = 0x108>>2; // Control List Executor Thread 0 End Address.
const V3D_CT1EA: usize   = 0x10c>>2; // Control List Executor Thread 1 End Address.
const V3D_CT0CA: usize   = 0x110>>2; // Control List Executor Thread 0 Current Address.
const V3D_CT1CA: usize   = 0x114>>2; // Control List Executor Thread 1 Current Address.
const V3D_CT00RA0: usize = 0x118>>2; // Control List Executor Thread 0 Return Address.
const V3D_CT01RA0: usize = 0x11c>>2; // Control List Executor Thread 1 Return Address.
const V3D_CT0LC: usize   = 0x120>>2; // Control List Executor Thread 0 List Counter
const V3D_CT1LC: usize   = 0x124>>2; // Control List Executor Thread 1 List Counter
const V3D_CT0PC: usize   = 0x128>>2; // Control List Executor Thread 0 Primitive List Counter
const V3D_CT1PC: usize   = 0x12c>>2; // Control List Executor Thread 1 Primitive List Counter

const V3D_PCS: usize     = 0x130>>2; // V3D Pipeline Control and Status
const V3D_BFC: usize     = 0x134>>2; // Binning Mode Flush Count
const V3D_RFC: usize     = 0x138>>2; // Rendering Mode Frame Count

//[Reserved; 1]
//[Reserved; 64]

const V3D_BPCA: usize    = 0x300>>2; // Current Address of Binning Memory Pool
const V3D_BPCS: usize    = 0x304>>2; // Remaining Size of Binning Memory Pool
const V3D_BPOA: usize    = 0x308>>2; // Address of Overspill Binning Memory Block
const V3D_BPOS: usize    = 0x30c>>2; // Size of Overspill Binning Memory Block
const V3D_BXCF: usize    = 0x310>>2; // Binner Debug

const V3D_SQRSV0: usize  = 0x410>>2; // Reserve QPUs 0-7
const V3D_SQRSV1: usize  = 0x414>>2; // Reserve QPUs 8-15
const V3D_SQCNTL: usize  = 0x418>>2; // QPU Scheduler Control

const V3D_SRQPC: usize   = 0x430>>2; // QPU User Program Request Program Address
const V3D_SRQUA: usize   = 0x434>>2; // QPU User Program Request Uniforms Address
const V3D_SRQUL: usize   = 0x438>>2; // QPU User Program Request Uniforms Length
const V3D_SRQCS: usize   = 0x43c>>2; // QPU User Program Request Control and Status

const V3D_VPACNTL: usize = 0x500>>2; // VPM Allocator Control
const V3D_VPMBASE: usize = 0x504>>2; // VPM base (user) memory reservation

const V3D_PCTRC: usize   = 0x670>>2; // Performance Counter Clear
const V3D_PCTRE: usize   = 0x674>>2; // Performance Counter Enables

const V3D_PCTR0: usize   = 0x680>>2; // Performance Counter Count 0
const V3D_PCTRS0: usize  = 0x684>>2; // Performance Counter Mapping 0
const V3D_PCTR1: usize   = 0x688>>2; // Performance Counter Count 1
const V3D_PCTRS1: usize  = 0x68c>>2; // Performance Counter Mapping 1
const V3D_PCTR2: usize   = 0x690>>2; // Performance Counter Count 2
const V3D_PCTRS2: usize  = 0x694>>2; // Performance Counter Mapping 2
const V3D_PCTR3: usize   = 0x698>>2; // Performance Counter Count 3
const V3D_PCTRS3: usize  = 0x69c>>2; // Performance Counter Mapping 3
const V3D_PCTR4: usize   = 0x6a0>>2; // Performance Counter Count 4
const V3D_PCTRS4: usize  = 0x6a4>>2; // Performance Counter Mapping 4
const V3D_PCTR5: usize   = 0x6a8>>2; // Performance Counter Count 5
const V3D_PCTRS5: usize  = 0x6ac>>2; // Performance Counter Mapping 5
const V3D_PCTR6: usize   = 0x6b0>>2; // Performance Counter Count 6
const V3D_PCTRS6: usize  = 0x6b4>>2; // Performance Counter Mapping 6
const V3D_PCTR7: usize   = 0x6b8>>2; // Performance Counter Count 7
const V3D_PCTRS7: usize  = 0x6bc>>2; // Performance Counter Mapping 7
const V3D_PCTR8: usize   = 0x6c0>>2; // Performance Counter Count 8
const V3D_PCTRS8: usize  = 0x6c4>>2; // Performance Counter Mapping 8
const V3D_PCTR9: usize   = 0x6c8>>2; // Performance Counter Count 9
const V3D_PCTRS9: usize  = 0x6cc>>2; // Performance Counter Mapping 9
const V3D_PCTR10: usize  = 0x6d0>>2; // Performance Counter Count 10
const V3D_PCTRS10: usize = 0x6d4>>2; // Performance Counter Mapping 10
const V3D_PCTR11: usize  = 0x6d8>>2; // Performance Counter Count 11
const V3D_PCTRS11: usize = 0x6dc>>2; // Performance Counter Mapping 11
const V3D_PCTR12: usize  = 0x6e0>>2; // Performance Counter Count 12
const V3D_PCTRS12: usize = 0x6e4>>2; // Performance Counter Mapping 12
const V3D_PCTR13: usize  = 0x6e8>>2; // Performance Counter Count 13
const V3D_PCTRS13: usize = 0x6ec>>2; // Performance Counter Mapping 13
const V3D_PCTR14: usize  = 0x6f0>>2; // Performance Counter Count 14
const V3D_PCTRS14: usize = 0x6f4>>2; // Performance Counter Mapping 14
const V3D_PCTR15: usize  = 0x6f8>>2; // Performance Counter Count 15
const V3D_PCTRS15: usize = 0x6fc>>2; // Performance Counter Mapping 15

const V3D_DBGE: usize    = 0xf00>>2; // PSE Error Signals
const V3D_FDBGO: usize   = 0xf04>>2; // FEP Overrun Error Signals
const V3D_FDBGB: usize   = 0xf08>>2; // FEP Interface Ready and Stall Signals, FEP Busy Signals
const V3D_FDBGR: usize   = 0xf0c>>2; // FEP Internal Ready Signals
const V3D_FDBGS: usize   = 0xf10>>2; // FEP Internal Stall Input Signals

//[Reserved; 3]
const V3D_ERRSTAT: usize = 0xf20>>2; // Miscellaneous Error Signals (VPM, VDW, VCD, VCM, L2C)



/// primitive typoe in the GL pipline
enum Primitive {
    POINT = 0,
    LINE = 1,
    LINE_LOOP = 2,
    LINE_STRIP = 3,
    TRIANGLE = 4,
    TRIANGLE_STRIP = 5,
    TRIANGLE_FAN = 6,
}

/// GL pipe control commands
#[repr(u8)]
pub enum Packet {
    HALT = 0,
    NOP = 1,

    MARKER = 2,
    RESET_MARKER_COUNT = 3,

    FLUSH = 4,
    FLUSH_ALL = 5,
    START_TILE_BINNING = 6,
    INCREMENT_SEMAPHORE = 7,
    WAIT_ON_SEMAPHORE = 8,

    BRANCH = 16,
    BRANCH_TO_SUB_LIST = 17,
    RETURN_FROM_SUBLIST = 18,

    REPEAT_START_MARKER = 19,
    REPEAT_FROM_START_MARKER = 20,

    STORE_MS_TILE_BUFFER = 24,
    STORE_MS_TILE_BUFFER_AND_EOF = 25,
    STORE_FULL_RES_TILE_BUFFER = 26,
    LOAD_FULL_RES_TILE_BUFFER = 27,
    STORE_TILE_BUFFER_GENERAL = 28,
    LOAD_TILE_BUFFER_GENERAL = 29,

    GL_INDEXED_PRIMITIVE = 32,
    GL_ARRAY_PRIMITIVE = 33,
    VG_COORDINATE_ARRAY_PRIMITIVES = 41,

    VG_INLINE_PRIMS = 42,

    COMPRESSED_PRIMITIVE = 48,
    CLIPPED_COMPRESSED_PRIMITIVE = 49,

    PRIMITIVE_LIST_FORMAT = 56,

    GL_SHADER_STATE = 64,
    NV_SHADER_STATE = 65,
    VG_SHADER_STATE = 66,
    VG_INLINE_SHADER_RECORD = 67,

    STATE_BLEND_MODE = 84,
    STATE_BLEND_CCOLOR_RGBA32 = 85,
    STATE_BLEND_CCOLOR_HDR16 = 86,

    CONFIGURATION_BITS = 96,
    FLAT_SHADE_FLAGS = 97,
    POINT_SIZE = 98,
    LINE_WIDTH = 99,
    RHT_X_BOUNDARY = 100,
    DEPTH_OFFSET = 101,
    CLIP_WINDOW = 102,
    VIEWPORT_OFFSET = 103,
    Z_CLIPPING = 104,
    CLIPPER_XY_SCALING = 105,
    CLIPPER_Z_SCALING = 106,

    TILE_BINNING_MODE_CONFIG = 112,
    TILE_RENDERING_MODE_CONFIG = 113,
    CLEAR_COLORS = 114,
    TILE_COORDINATES = 115,

    // Not an actual hardware packet -- this is what we use to put
    // references to GEM bos in the command stream, since we need the u32
    // int the actual address packet in order to store the offset from the
    // start of the BO.
    //VC4_PACKET_GEM_HANDLES = 254,
}

pub fn InitV3D() -> Result<(), ()> {
    let tag = mbox::Mailbox::new().tag_message(&[
        mbox::Tag::SET_CLOCK_RATE as u32, 8, 8, mbox::ClockId::V3D as u32, 250000000,
        mbox::Tag::ENABLE_QPU as u32, 4, 4, 1,
    ]).is_some();
    unsafe {
        if tag && v3d.offset(V3D_IDENT0 as isize).read().read() == 0x02443356 {
            Ok(())
        } else {
            Err(())
        }
    }
}

unsafe fn run_a(msg: &[u32]) -> Option<u32> {
    mbox::Mailbox::new().tag_message(msg).map(|buf| buf[3+3])
}
unsafe fn run_b(msg: &[u32]) -> bool {
    mbox::Mailbox::new().tag_message(msg).is_some()
}

unsafe fn exec_code(code: u32, r0: u32, r1: u32, r2: u32, r3: u32, r4: u32, r5: u32) -> bool {
    run_b(&[mbox::Tag::EXECUTE_CODE as u32, 28, 28, code, r0, r1, r2, r3, r4, r5])
}
unsafe fn exec_qpu(num_qpus: i32, control: u32, noflush: u32, timeout: u32) -> bool {
    run_b(&[mbox::Tag::EXECUTE_QPU as u32, 28, 28, num_qpus as u32, control, noflush, timeout])
}



/// Flags for allocate memory.
bitflags! {
    struct MemFlags: u32 {
        /// can be resized to 0 at any time.
        /// Use for cached data
        const DISCARDABLE = 1 << 0;

        /// normal allocating alias.
        /// Don't use from ARM
        const NORMAL = 0 << 2;
        /// 0xC alias uncached
        const DIRECT = 1 << 2;

        /// 0x8 alias.
        /// Non-allocating in L2 but coherent
        const COHERENT = 2 << 2;

        /// initialise buffer to all zeros
        const ZERO = 1 << 4;

        /// don't initialise (default is initialise to all ones)
        const NO_INIT = 1 << 5;

        /// Likely to be locked for long periods of time.
        const HINT_PERMALOCK = 1 << 6;

        /// Allocating in L2
        const L1_NONALLOCATING = Self::DIRECT.bits | Self::COHERENT.bits;
    }
}

struct Memory {
    handle: u32,
    size: u32,
    align: u32,
    flags: MemFlags,
}

impl Memory {
    fn alloc(size: u32, align: u32, flags: MemFlags) -> Option<Self> {
        unsafe {
            run_a(&[mbox::Tag::ALLOCATE_MEMORY as u32, 12, 12, size, align, flags.bits])
                .map(|handle| Self { handle, size, align, flags })
        }
    }
}

impl Drop for Memory {
    fn drop(&mut self) {
        unsafe {
            run_b(&[mbox::Tag::RELEASE_MEMORY as u32, 4, 4, self.handle]);
        }
    }
}

struct Lock<'a> {
    handle: u32,
    mem: &'a mut Memory,
}

impl<'a> Lock<'a> {
    fn lock(mem: &'a mut Memory) -> Option<Self> {
        unsafe {
            run_a(&[mbox::Tag::LOCK_MEMORY as u32, 4, 4, mem.handle])
                .map(move |handle| Self { handle, mem })
        }
    }

    fn slice(&mut self) -> &mut [u8] {
        unsafe {
            let p = gpu2arm(self.handle as usize) as *mut u8;
            let size = self.mem.size as usize;
            ::std::slice::from_raw_parts_mut(p, size)
        }
    }
}

impl<'a> Drop for Lock<'a> {
    fn drop(&mut self) {
        unsafe {
            //XXX self.run_b(&[mbox::Tag::UNLOCK_MEMORY as u32, 4, 4, self.handle])
            run_b(&[mbox::Tag::UNLOCK_MEMORY as u32, 4, 4, self.mem.handle]);
        }
    }
}



#[repr(align(1))]
#[derive(Clone, Copy)]
struct Data {
    byte1: u8,
    byte2: u8,
    byte3: u8,
    byte4: u8,
}

union Emit {
    d: Data,
    u1: u8,
    u2: u16,
    u3: u32,
    f: f32,
}

struct Buffer<'a> {
    p: *mut u8,
    slice: &'a mut [u8],
}

macro_rules! api {
    ($key:ident => $fn:ident ($( $arg:ident : $arg_t:ident ),*) ) => {
        fn $fn(&mut self, $($arg: $arg_t),*) {
            self.u8(Packet::$key as u8);
            $(
                self.$arg_t($arg);
            )*
        }
    }
}


impl<'a> Buffer<'a> {

    fn start(slice: &'a mut [u8]) -> Self {
        let p = slice.as_mut_ptr();
        Self { p, slice }
    }

    fn end(self) -> usize {
        let Self { p, slice } = self;
        p as usize - slice.as_mut_ptr() as usize
    }

    fn u8(&mut self, d: u8) {
        unsafe {
            self.p.write_volatile(d);
            self.p = self.p.add(1);
        }
    }
    fn u16(&mut self, u2: u16) {
        let d = Emit { u2 };
        unsafe {
            self.u8(d.d.byte1);
            self.u8(d.d.byte2);
        }
    }
    fn u32(&mut self, u3: u32) {
        let d = Emit { u3 };
        unsafe {
            self.u8(d.d.byte1);
            self.u8(d.d.byte2);
            self.u8(d.d.byte3);
            self.u8(d.d.byte4);
        }
    }
    fn f32(&mut self, f: f32) {
        let d = Emit { f };
        unsafe {
            self.u8(d.d.byte1);
            self.u8(d.d.byte2);
            self.u8(d.d.byte3);
            self.u8(d.d.byte4);
        }
    }

    api!(TILE_BINNING_MODE_CONFIG => tile_binning_mode_config(addr: u32, size: u32, data: u32, w: u8, h: u8, config: u8));

    api!(START_TILE_BINNING => start_tile_binning());
    api!(PRIMITIVE_LIST_FORMAT => primitive_list_format(flags: u8));
    api!(CLIP_WINDOW => clip_window(x: u16, y: u16, w: u16, h: u16));
    api!(CONFIGURATION_BITS => config_state(a: u8, b: u8, c: u8));
    api!(VIEWPORT_OFFSET => viewport_offset(x: u16, y: u16));
    api!(NV_SHADER_STATE => nv_shader_state(addr: u32));
    api!(GL_INDEXED_PRIMITIVE => gl_indexed_primitive(prim: u8, idx: u32, len: u32, max: u32));

    api!(FLUSH_ALL => flush_all());
    api!(NOP => nop());
    api!(HALT => halt());
}

// Render a single triangle to memory.
fn test_triangle(render_w: u16, render_h: u16, render_buffer_addr: u32/*, prn_handler: printhandler*/) {
    // We allocate/lock some videocore memory
    // I'm just shoving everything in a single buffer because I'm lazy 8Mb, 4k alignment
    // Normally you would do individual allocations but not sure of interface I want yet
    // So lets just define some address in that one single buffer for now
    // You need to make sure they don't overlap if you expand sample
    const BUFFER_VERTEX_INDEX: usize = 0x70;
    const BUFFER_SHADER_OFFSET: usize = 0x80;
    const BUFFER_VERTEX_DATA: usize = 0x100;
    const BUFFER_TILE_STATE: usize = 0x200;
    const BUFFER_TILE_DATA: usize = 0x6000;
    const BUFFER_RENDER_CONTROL: usize = 0xe200;
    const BUFFER_FRAGMENT_SHADER: usize = 0xfe00;
    const BUFFER_FRAGMENT_UNIFORM: usize = 0xff00;

    let mut handle = Memory::alloc(0x80_0000, 0x1000, MemFlags::COHERENT | MemFlags::ZERO)
        .expect("Unable to allocate memory");
    let mut bus = Lock::lock(&mut handle).expect("lock");
    let bus_addr = bus.handle as usize;

    let cmd_length = {
        let mut p = Buffer::start(bus.slice());

        // Configuration stuff
        // Tile Binning Configuration.
        //   Tile state data is 48 bytes per tile, I think it can be thrown away
        //   as soon as binning is finished.
        //   A tile itself is 64 x 64 pixels
        //   we will need binWth of them across to cover the render width
        //   we will need binHt of them down to cover the render height
        //   we add 63 before the division because any fraction at end must have a bin
        let bin_w = (render_w as u32 + 63) / 64;  // Tiles across
        let bin_h = (render_h as u32 + 63) / 64;    // Tiles down

        p.tile_binning_mode_config(
            (bus_addr + BUFFER_TILE_DATA) as u32,  // tile allocation memory address
            0x4000,                                // tile allocation memory size
            (bus_addr + BUFFER_TILE_STATE) as u32, // Tile state data address
            bin_w as u8, bin_h as u8, 0x04);      // render_w/64, render_h/64, config

        // Start tile binning
        p.start_tile_binning();

        p.primitive_list_format(0x32); // Primitive type: 16 bit triangle
        p.clip_window(0, 0, render_w, render_h);

        // enable both foward and back facing polygons
        // depth testing disabled
        // enable early depth write
        p.config_state(0x03, 0x00, 0x02);

        p.viewport_offset(0, 0);

        // The triangle

        // No Vertex Shader state
        // (takes pre-transformed vertexes so we don't have to supply a working coordinate shader.)
        p.nv_shader_state((bus_addr + BUFFER_SHADER_OFFSET) as u32);

        // 8bit index, triangles
        p.gl_indexed_primitive(Primitive::TRIANGLE as u8, 9, (bus_addr + BUFFER_VERTEX_INDEX) as u32, 6);

        // End of bin list
        // So Flush, Nop and Halt
        p.flush_all();
        p.nop();
        p.halt();

        p.end()
    };


    // Okay now we need Shader Record to buffer
    {
        let mut p = Buffer::start(&mut bus.slice()[BUFFER_SHADER_OFFSET..]);
        p.u8(0x01);                                         // flags
        p.u8(6 * 4);                                        // stride
        p.u8(0xcc);                                         // num uniforms (not used)
        p.u8(3);                                            // num varyings
        p.u32((bus_addr + BUFFER_FRAGMENT_SHADER) as u32);  // Fragment shader code
        p.u32((bus_addr + BUFFER_FRAGMENT_UNIFORM) as u32); // Fragment shader uniforms
        p.u32((bus_addr + BUFFER_VERTEX_DATA) as u32);      // Vertex Data
    }


    // Vertex Data
    {
        let mut p = Buffer::start(&mut bus.slice()[BUFFER_VERTEX_DATA..]);

        // Setup triangle vertices from OpenGL tutorial which used this
        // [0] = -0.4; [1] = 0.1; [2] = 0.0;
        // [3] =  0.4; [4] = 0.1; [5] = 0.0;
        // [6] =  0.0; [7] = 0.7; [8] = 0.0;
        let cx = (render_w / 2) as u16;
        let cy = (0.4 * (render_h as f32 / 2.0)) as u16;
        let half_w = (0.4 * (render_w as f32 / 2.0)) as u16;
        let half_h = (0.3 * (render_h as f32 / 2.0)) as u16;

        // Vertex: Top, vary red
        p.u16((cx         ) << 4);          // X in 12.4 fixed point
        p.u16((cy - half_h) << 4);          // Y in 12.4 fixed point
        p.f32(1.0); p.f32(1.0);             // Z; 1/W
        p.f32(1.0); p.f32(0.0); p.f32(0.0); // Varying RGB

        // Vertex: bottom left, vary blue
        p.u16((cx - half_w) << 4);          // X in 12.4 fixed point
        p.u16((cy + half_h) << 4);          // Y in 12.4 fixed point
        p.f32(1.0); p.f32(1.0);             // Z; 1/W
        p.f32(0.0); p.f32(0.0); p.f32(1.0); // Varying RGB

        // Vertex: bottom right, vary green
        p.u16((cx + half_w) << 4);                // X in 12.4 fixed point
        p.u16((cy + half_h) << 4);                // Y in 12.4 fixed point
        p.f32(1.0); p.f32(1.0);             // Z; 1/W
        p.f32(0.0); p.f32(1.0); p.f32(0.0); // Varying RGB

        // Setup triangle vertices (for quad) from OpenGL tutorial which used this
        // [0] = -0.2; [ 1] = -0.1; [ 2] = 0.0;
        // [3] = -0.2; [ 4] = -0.6; [ 5] = 0.0;
        // [6] =  0.2; [ 7] = -0.1; [ 8] = 0.0;
        // [9] =  0.2; [10] = -0.6; [11] = 0.0;
        let cy = (1.35 * (render_h as f32 / 2.0)) as u16;

        // Vertex: Top, left  vary blue
        p.u16((cx - half_w) << 4);          // X in 12.4 fixed point
        p.u16((cy - half_h) << 4);          // Y in 12.4 fixed point
        p.f32(1.0); p.f32(1.0);             // Z; 1/W
        p.f32(0.0); p.f32(0.0); p.f32(1.0); // Varying RGB

        // Vertex: bottom left, vary Green
        p.u16((cx - half_w) << 4);          // X in 12.4 fixed point
        p.u16((cy + half_h) << 4);          // Y in 12.4 fixed point
        p.f32(1.0); p.f32(1.0);             // Z; 1/W
        p.f32(0.0); p.f32(1.0); p.f32(0.0); // Varying RGB

        // Vertex: top right, vary red
        p.u16((cx + half_w) << 4);          // X in 12.4 fixed point
        p.u16((cy - half_h) << 4);          // Y in 12.4 fixed point
        p.f32(1.0); p.f32(1.0);             // Z; 1/W
        p.f32(1.0); p.f32(0.0); p.f32(0.0); // Varying RGB

        // Vertex: bottom right, vary yellow
        p.u16((cx + half_w) << 4);          // X in 12.4 fixed point
        p.u16((cy + half_h) << 4);          // Y in 12.4 fixed point
        p.f32(1.0); p.f32(1.0);             // Z; 1/W
        p.f32(0.0); p.f32(1.0); p.f32(1.0); // Varying RGB
    }

    // Vertex list
    {
        let mut p = Buffer::start(&mut bus.slice()[BUFFER_VERTEX_INDEX..]);
        // tri  - top bottom_left bottom_right
        // quad - top_left bottom_left top_right
        // quad - bottom_left bottom_right top_right
        for i in 0..6 {
            p.u8(i);
        }
    }

    // fragment shader
    {
        let mut p = Buffer::start(&mut bus.slice()[BUFFER_FRAGMENT_SHADER..]);
        p.u32(0x958e0dbf);
        p.u32(0xd1724823); // mov r0, vary; mov r3.8d, 1.0
        p.u32(0x818e7176);
        p.u32(0x40024821); // fadd r0, r0, r5; mov r1, vary
        p.u32(0x818e7376);
        p.u32(0x10024862); // fadd r1, r1, r5; mov r2, vary
        p.u32(0x819e7540);
        p.u32(0x114248a3); // fadd r2, r2, r5; mov r3.8a, r0
        p.u32(0x809e7009);
        p.u32(0x115049e3); // nop; mov r3.8b, r1
        p.u32(0x809e7012);
        p.u32(0x116049e3); // nop; mov r3.8c, r2
        p.u32(0x159e76c0);
        p.u32(0x30020ba7); // mov tlbc, r3; nop; thrend
        p.u32(0x009e7000);
        p.u32(0x100009e7); // nop; nop; nop
        p.u32(0x009e7000);
        p.u32(0x500009e7); // nop; nop; sbdone
    }

        /*
    // Render control list
    p = list + BUFFER_RENDER_CONTROL;

    // Clear colors
    emit_uint8_t(&p, GL_CLEAR_COLORS);
    emit_uint32_t(&p, 0xff000000);            // Opaque Black
    emit_uint32_t(&p, 0xff000000);            // 32 bit clear colours need to be repeated twice
    emit_uint32_t(&p, 0);
    emit_uint8_t(&p, 0);

    // Tile Rendering Mode Configuration
    emit_uint8_t(&p, GL_TILE_RENDER_CONFIG);

    emit_uint32_t(&p, renderBufferAddr);    // render address

    emit_uint16_t(&p, renderWth);            // width
    emit_uint16_t(&p, renderHt);            // height
    emit_uint8_t(&p, 0x04);                    // framebuffer mode (linear rgba8888)
    emit_uint8_t(&p, 0x00);

    // Do a store of the first tile to force the tile buffer to be cleared
    // Tile Coordinates
    emit_uint8_t(&p, GL_TILE_COORDINATES);
    emit_uint8_t(&p, 0);
    emit_uint8_t(&p, 0);
    // Store Tile Buffer General
    emit_uint8_t(&p, GL_STORE_TILE_BUFFER);
    emit_uint16_t(&p, 0);                    // Store nothing (just clear)
    emit_uint32_t(&p, 0);                    // no address is needed

    // Link all binned lists together
    for (int x = 0; x < binWth; x++) {
        for (int y = 0; y < binHt; y++) {

            // Tile Coordinates
            emit_uint8_t(&p, GL_TILE_COORDINATES);
            emit_uint8_t(&p, x);
            emit_uint8_t(&p, y);

            // Call Tile sublist
            emit_uint8_t(&p, GL_BRANCH_TO_SUBLIST);
            emit_uint32_t(&p, bus_addr + BUFFER_TILE_DATA + (y * binWth + x) * 32);

            // Last tile needs a special store instruction
            if (x == (binWth - 1) && (y == binHt - 1)) {
                // Store resolved tile color buffer and signal end of frame
                emit_uint8_t(&p, GL_STORE_MULTISAMPLE_END);
            } else {
                // Store resolved tile color buffer
                emit_uint8_t(&p, GL_STORE_MULTISAMPLE);
            }
        }
    }


    int render_length = p - (list + BUFFER_RENDER_CONTROL);


    // Run our control list
    v3d[V3D_BFC] = 1;                         // reset binning frame count
    v3d[V3D_CT0CA] = bus_addr;
    v3d[V3D_CT0EA] = bus_addr + length;

    // Wait for control list to execute
    while (v3d[V3D_CT0CS] & 0x20);

    // wait for binning to finish
    while (v3d[V3D_BFC] == 0) {}

    // stop the thread
    v3d[V3D_CT0CS] = 0x20;

    // Run our render
    v3d[V3D_RFC] = 1;                        // reset rendering frame count
    v3d[V3D_CT1CA] = bus_addr + BUFFER_RENDER_CONTROL;
    v3d[V3D_CT1EA] = bus_addr + BUFFER_RENDER_CONTROL + render_length;

    // Wait for render to execute
    while (v3d[V3D_CT1CS] & 0x20);

    // wait for render to finish
    while(v3d[V3D_RFC] == 0) {}

    // stop the thread
    v3d[V3D_CT1CS] = 0x20;

    // Release resources
    V3D_mem_unlock(handle);
    V3D_mem_free(handle);
*/
}
