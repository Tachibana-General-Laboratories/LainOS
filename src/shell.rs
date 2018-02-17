use stack_vec::StackVec;
use std::str::from_utf8;

use pi::uart0;
use pi::power;
use util;

/// Error type for `Command` parse failures.
#[derive(Debug)]
enum Error {
    Empty,
    TooManyArgs
}

/// A structure representing a single shell command.
struct Command<'a> {
    args: StackVec<'a, &'a str>
}

impl<'a> Command<'a> {
    /// Parse a command from a string `s` using `buf` as storage for the
    /// arguments.
    ///
    /// # Errors
    ///
    /// If `s` contains no arguments, returns `Error::Empty`. If there are more
    /// arguments than `buf` can hold, returns `Error::TooManyArgs`.
    fn parse(s: &'a str, buf: &'a mut [&'a str]) -> Result<Command<'a>, Error> {
        let mut args = StackVec::new(buf);
        for arg in s.split(' ').filter(|a| !a.is_empty()) {
            args.push(arg).map_err(|_| Error::TooManyArgs)?;
        }

        if args.is_empty() {
            return Err(Error::Empty);
        }

        Ok(Command { args })
    }

    /// Returns this command's path. This is equivalent to the first argument.
    fn path(&self) -> &str {
        &self.args[0]
    }
}

/// Starts a shell using `prefix` as the prefix for each line. This function
/// never returns: it is perpetually in a shell loop.
pub fn shell(prefix: &str) -> ! {
    print!("\n{}", prefix);

    let mut buf = [0u8; 512];
    let mut buf = StackVec::new(&mut buf);
    loop {
        match uart0::receive() {
            0 => (),
            b'\r' | b'\n' => {
                print!("\r\n");
                {
                    let s = from_utf8(&buf).unwrap();
                    let mut str_buf = [""; 64];
                    match Command::parse(s, &mut str_buf) {
                        Err(Error::Empty) => (),
                        Err(Error::TooManyArgs) => println!("error: too many arguments"),
                        Ok(cmd) => {
                            run_cmd(cmd);
                            print!("\r\n");
                        }
                    }
                }
                buf.truncate(0);
                print!("{}", prefix);
            }
            127 => (), // DEL
            8 => { // BS
                if !buf.is_empty() {
                    uart0::send(8);
                    uart0::send(32);
                    uart0::send(8);
                    buf.pop();
                }
            }
            c @ 32...126 => {
                if !buf.is_full() {
                    buf.push(c).unwrap();
                    uart0::send(c);
                }
            }
            _ => uart0::send(7), // send bell
        }
    }
}

fn run_cmd(cmd: Command) {
    match cmd.path() {
        "echo" => echo(cmd.args),
        "ls" => ls(cmd.args),
        "dump" => dump(cmd.args),
        "poweroff" => {
            print!("power-off the machine\n");
            power::power_off();
        }
        "reset" => {
            print!("reset the machine\n");
            power::reset();
        }
        _ => print!("unknown command: {}", cmd.path()),
    }
}

fn echo<'a>(args: StackVec<'a, &'a str>) {
    for (i, arg) in args.iter().enumerate() {
        match i {
            0 => (),
            1 => print!("{}", arg),
            _ => print!(" {}", arg),
        }
    }
}

fn ls<'a>(args: StackVec<'a, &'a str>) {
    let dir = if args.len() > 1 {
        args[1]
    } else {
        ""
    };
    println!("  no file system yet;   but... maybe {} is:", dir);
    match dir {
        "/" => print!("bin etc sys usr var"),
        _ => print!("ls: cannot access '{}': No such file or directory", dir),
    }
}


fn dump<'a>(args: StackVec<'a, &'a str>) {
    if args.len() < 2 {
        println!("usage:");
        print!  ("    dump <hex addr> <size=512>");
        return;
    }

    let addr = usize::from_str_radix(args[1], 16).unwrap_or(0x80_0000);
    let size = if args.len() > 2 {
        usize::from_str_radix(args[2], 10).unwrap_or(256)
    } else {
        256
    };

    util::dump(unsafe { addr as *const u8 }, size);
}
