use stack_vec::StackVec;
use std::str::from_utf8;

use pi::power;
use console::{kprint, kprintln, CONSOLE};

use std::io;

use std::path::{Path, PathBuf};

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

    let mut shell = Shell { cwd: PathBuf::from("/") };

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
                        Ok(cmd) => shell.run(cmd),
                    }
                }
                buf.truncate(0);
                kprint!("{}", prefix);
            }
            127 | 8 => { // DEL | BS
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
    cwd: PathBuf,
}

impl Shell {
    fn run(&mut self, cmd: Command) {
        match cmd.path() {
            "echo" => {
                self.echo(cmd.args);
                kprint!("\r\n");
            }
            "pwd" => {
                kprint!("{}", self.cwd.to_str().unwrap());
                kprint!("\r\n");
            }
            "cd" => self.cd(cmd.args),
            "ls" => self.ls(cmd.args),
            "cat" => self.cat(cmd.args),

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
            _ => kprintln!("unknown command: {}", cmd.path()),
        }
    }

    fn canonicalize<P: AsRef<Path>>(&self, p: P) -> io::Result<PathBuf> {
        use std::path::Component::*;

        let mut result = PathBuf::new();
        for (i, c) in p.as_ref().components().enumerate() {
            match c {
            Prefix(_) => unimplemented!(),
            CurDir => result.push(&self.cwd),
            RootDir => result.push(RootDir),
            ParentDir if i == 0 => {
                if let Some(parent) = self.cwd.parent() {
                    result.push(parent)
                } else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "fail cwd.parent dir"));
                }
            }
            ParentDir => {
                if !result.pop() {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "fail parent dir"));
                }
            }
            Normal(s) => result.push(s),
            }
        }

        Ok(result)
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
        // TODO: "." and ".." support
        if args.len() == 1 {
            self.cwd = PathBuf::from("/");
        } else {
            let mut dir = self.cwd.clone();
            dir.push(args[1]);
            let dir = self.canonicalize(&dir)
                .and_then(|dir| FILE_SYSTEM.open_dir(&dir).map(|_| dir));
            match dir {
                Ok(dir) => self.cwd = dir,
                Err(err) => kprintln!("cd: {:?}", err),
            }
        }
    }

    fn ls(&mut self, args: StackVec<&str>) {
        let mut dir = self.cwd.clone();
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

    fn cat(&mut self, args: StackVec<&str>) {
        if args.len() == 1 {
            return
        }
        let mut dir = self.cwd.clone();
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
