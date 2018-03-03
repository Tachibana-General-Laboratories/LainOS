use stack_vec::StackVec;
use std::str::from_utf8;

use pi::power;
use util;
use console::{kprint, kprintln, CONSOLE};

use std::io::{self, Write, Read};

use std::path::PathBuf;

use super::FILE_SYSTEM;
use fat32::traits::*;

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
    use std::io::Write;
    kprint!("\n{}", prefix);

    let working_directory = PathBuf::from("/");
    let mut shell = Shell { working_directory };

    let mut buf = [0u8; 512];
    let mut buf = StackVec::new(&mut buf);
    loop {
        let byte = CONSOLE.lock().read_byte();
        match byte {
            0 => (),
            b'\r' | b'\n' => {
                kprint!("\r\n");
                {
                    let s = from_utf8(&buf).unwrap();
                    let mut str_buf = [""; 64];
                    match Command::parse(s, &mut str_buf) {
                        Err(Error::Empty) => (),
                        Err(Error::TooManyArgs) => kprintln!("error: too many arguments"),
                        Ok(cmd) => {
                            shell.run(cmd);
                            kprint!("\r\n");
                        }
                    }
                }
                buf.truncate(0);
                kprint!("{}", prefix);
            }
            127 => kprintln!("DEL"), // DEL
            8 => { // BS
                if !buf.is_empty() {
                    CONSOLE.lock().write(&[8, 32, 8]).unwrap();
                    buf.pop();
                }
            }
            c @ 32...126 => {
                if !buf.is_full() {
                    buf.push(c).unwrap();
                    CONSOLE.lock().write(&[c]).unwrap();
                }
            }
            _ => {
                // send bell
                CONSOLE.lock().write(&[7]).unwrap();
            }
        }
    }
}

struct Shell {
    working_directory: PathBuf,
}

impl Shell {
    fn run(&mut self, cmd: Command) {
        match cmd.path() {
            "pwd" => self.pwd(cmd.args),
            "cd" => self.cd(cmd.args),
            "ls" => self.ls(cmd.args),
            "cat" => self.cat(cmd.args),

            "echo" => self.echo(cmd.args),
            "poweroff" => {
                kprintln!("power-off the machine");
                power::power_off();
            }
            "halt" => {
                kprintln!("halt the machine");
                power::halt();
            }
            "reset" => {
                kprintln!("reset the machine");
                power::reset();
            }
            _ => kprint!("unknown command: {}", cmd.path()),
        }
    }

    fn echo(&self, args: StackVec<&str>) {
        for (i, arg) in args.iter().enumerate() {
            match i {
                0 => (),
                1 => kprint!("{}", arg),
                _ => kprint!(" {}", arg),
            }
        }
    }

    fn cd(&mut self, args: StackVec<&str>) {
        if args.len() == 1 {
            self.working_directory = PathBuf::from("/");
        } else {
            let mut dir = self.working_directory.clone();
            dir.push(args[1]);
            match FILE_SYSTEM.open_dir(&dir) {
                Ok(e) => self.working_directory = dir,
                Err(err) => kprintln!("cd: {:?}", err),
            }
        }
    }

    fn ls(&mut self, args: StackVec<&str>) {
        let mut dir = self.working_directory.clone();
        if args.len() > 1 {
            dir.push(args[1])
        }
        match FILE_SYSTEM.open_dir(&dir).and_then(|e| e.entries()) {
            Ok(entries) => {
                for e in entries {
                    kprintln!("{}", e.name());
                }
            }
            Err(err) => kprintln!("ls: {}", err),
        }
    }

    fn pwd(&mut self, args: StackVec<&str>) {
        kprint!("{}", self.working_directory.to_str().unwrap());
    }

    fn cat(&mut self, args: StackVec<&str>) {
        if args.len() == 1 {
            return
        }
        let mut dir = self.working_directory.clone();
        dir.push(args[1]);

        match FILE_SYSTEM.open_file(&dir) {
            Ok(mut r) => {
                let mut w = CONSOLE.lock();
                io::copy(&mut r, &mut *w).unwrap();
            }
            Err(err) => kprintln!("cat: {}", err),
        }
    }
}
