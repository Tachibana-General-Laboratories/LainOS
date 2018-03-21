extern crate serial;
extern crate structopt;
extern crate xmodem;
#[macro_use] extern crate structopt_derive;

use std::path::PathBuf;
use std::time::Duration;
use std::io::{self, Read, Write, BufReader, BufRead};
use std::fs::File;

use structopt::StructOpt;
use serial::core::{CharSize, BaudRate, StopBits, FlowControl, SerialDevice, SerialPortSettings};
use xmodem::{Xmodem, Progress};

mod parsers;

use parsers::{parse_width, parse_stop_bits, parse_flow_control, parse_baud_rate};

#[derive(StructOpt, Debug)]
#[structopt(about = "Write to TTY using the XMODEM protocol by default.")]
struct Opt {
    #[structopt(short = "i", help = "Input file (defaults to stdin if not set)", parse(from_os_str))]
    input: Option<PathBuf>,

    #[structopt(short = "b", long = "baud", parse(try_from_str = "parse_baud_rate"),
                help = "Set baud rate", default_value = "115200")]
    baud_rate: BaudRate,

    #[structopt(short = "t", long = "timeout", parse(try_from_str),
                help = "Set timeout in seconds", default_value = "10")]
    timeout: u64,

    #[structopt(short = "w", long = "width", parse(try_from_str = "parse_width"),
                help = "Set data character width in bits", default_value = "8")]
    char_width: CharSize,

    #[structopt(help = "Path to TTY device", parse(from_os_str))]
    tty_path: PathBuf,

    #[structopt(short = "f", long = "flow-control", parse(try_from_str = "parse_flow_control"),
                help = "Enable flow control ('hardware' or 'software')", default_value = "none")]
    flow_control: FlowControl,

    #[structopt(short = "s", long = "stop-bits", parse(try_from_str = "parse_stop_bits"),
                help = "Set number of stop bits", default_value = "1")]
    stop_bits: StopBits,

    #[structopt(short = "r", long = "raw", help = "Disable XMODEM")]
    raw: bool,
}

fn main() {
    let opt = Opt::from_args();
    let mut serial = serial::open(&opt.tty_path).expect("path points to invalid TTY");

    {
        let mut settings = serial.read_settings().unwrap();
        settings.set_baud_rate(opt.baud_rate).unwrap();
        settings.set_char_size(opt.char_width);
        settings.set_stop_bits(opt.stop_bits);
        settings.set_flow_control(opt.flow_control);
        serial.write_settings(&settings).unwrap();
    }

    serial.set_timeout(Duration::from_secs(opt.timeout)).unwrap();

    if let Some(input) = opt.input {
        let input = BufReader::new(File::open(input).unwrap());
        transmit(opt.raw, input, serial).unwrap();
    } else {
        let input = BufReader::new(io::stdin());
        transmit(opt.raw, input, serial).unwrap();
    }

    fn transmit<R, W>(raw: bool, mut data: R, mut to: W) -> io::Result<()>
        where W: Read + Write, R: BufRead
    {
        if raw {
            io::copy(&mut data, &mut to).map(|_| ())
        } else {
            Xmodem::transmit(&mut data, &mut to).map(|_| ())
        }
    }
}
