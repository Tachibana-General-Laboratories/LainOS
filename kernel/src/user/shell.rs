use sys::StackVec;
use core::str::from_utf8;
use core::str::FromStr;

use super::syscall::*;

//use pi::power;
//use console::CONSOLE;

//use std::io;

//use std::path::{Path, PathBuf};

//use FILE_SYSTEM;
//use sys::fs::*;
//

fn read_byte() -> u8 {
    syscall_read_byte().unwrap()
}

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
pub fn shell(prefix: &str) -> u32 {
    //use std::io::Write;

    println!("");
    println!("      .  ");
    println!("    < 0 >");
    println!("    ./ \\.");
    println!("");

    //print!("(/) {}", prefix);
    print!("{}", prefix);

    //let mut shell = Shell { cwd: PathBuf::from("/") };
    let mut shell = Shell { exit: None };

    let mut buf = [0u8; 512];
    let mut buf = StackVec::new(&mut buf);
    while shell.exit.is_none() {
        let byte = read_byte();
        match byte {
            0 => (),
            b'\r' | b'\n' => {
                print!("\r\n");
                {
                    let s = from_utf8(&buf).unwrap();
                    let mut str_buf = [""; 64];
                    match Command::parse(s, &mut str_buf) {
                        Err(Error::Empty) => (),
                        Err(Error::TooManyArgs) => println!("error: too many arguments"),
                        Ok(cmd) => shell.run(cmd),
                    }
                }
                buf.truncate(0);
                print!("{}", prefix);
                //let cwd = shell.cwd.to_str().unwrap();
                //print!("({}) {}", cwd, prefix);
            }
            127 | 8 => { // DEL | BS
                if !buf.is_empty() {
                    write_buf(&[8, 32, 8]);
                    buf.pop();
                }
            }
            c @ 32...126 => {
                if !buf.is_full() {
                    buf.push(c).unwrap();
                    write_buf(&[c]);
                }
            }
            _ => {
                // send bell
                write_buf(&[7]);
            }
        }
    }

    shell.exit.unwrap()
}

struct Shell {
    //cwd: PathBuf,
    exit: Option<u32>,
}

impl Shell {
    fn run(&mut self, cmd: Command) {
        match cmd.path() {
            "echo" => {
                self.echo(cmd.args);
                print!("\r\n");
            }
            "exit" => {
                let code: u32 = cmd.args.get(1)
                    .and_then(|s| u32::from_str(s).ok())
                    .unwrap_or(0);
                self.exit = Some(code);
                print!("\r\n");
            }
            /*
            "pwd" => {
                print!("{}", self.cwd.to_str().unwrap());
                print!("\r\n");
            }
            "cd" => self.cd(cmd.args),
            "ls" => self.ls(cmd.args),
            "cat" => self.cat(cmd.args),
            */

            /*
            "poweroff" => {
                println!("power-off the machine");
                power::power_off();
            }
            "halt" => {
                println!("halt the machine");
                power::halt();
            }
            "reset" => {
                println!("reset the machine");
                power::reset();
            }
            */
            _ => println!("unknown command: {}", cmd.path()),
        }
    }
    /*

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
    */

    fn echo(&self, args: StackVec<&str>) {
        for (i, arg) in args.iter().enumerate() {
            match i {
                0 => (),
                1 => print!("{}", arg),
                _ => print!(" {}", arg),
            }
        }
    }

    /*
    fn cd(&mut self, args: StackVec<&str>) {
        if args.len() == 1 {
            self.cwd = PathBuf::from("/");
        } else {
            let mut dir = self.cwd.clone();
            dir.push(args[1]);
            let dir = self.canonicalize(&dir)
                .and_then(|dir| FILE_SYSTEM.open_dir(&dir).map(|_| dir));
            match dir {
                Ok(dir) => self.cwd = dir,
                Err(err) => println!("cd: {:?}", err),
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
                    println!("{}", e.name());
                }
            }
            Err(err) => println!("ls: {}", err),
        }
    }

    fn cat(&mut self, args: StackVec<&str>) {
        if args.len() == 1 {
            return
        }
        for arg in &args[1..] {
            let mut dir = self.cwd.clone();
            dir.push(arg);
            match FILE_SYSTEM.open_file(&dir) {
                Ok(mut r) => {
                    let mut w = CONSOLE.lock();
                    io::copy(&mut r, &mut *w).unwrap();
                }
                Err(err) => println!("cat: {}", err),
            }
        }
    }
    */
}
