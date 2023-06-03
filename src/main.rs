use std::{
    fs,
    io::{self, stdin, stdout, BufReader, BufWriter, Read, Write},
};

use clap::Parser;
use cli::Cli;

mod cli;

#[cfg(test)]
mod test;

const INVALID_ASCII: u8 = b'.';

/// This takes `&mut [u8]` to modify the slice rather than alloc a new one
fn render_ascii(w: &mut impl Write, bytes: &mut [u8]) -> io::Result<()> {
    let mut i = 0;
    while i < bytes.len() {
        let b = &mut bytes[i];
        if !(b'!' <= *b && *b <= b'~') && *b != b' ' {
            *b = INVALID_ASCII as u8;
        }
        i += 1;
    }
    w.write(bytes)?;
    Ok(())
}

const HEX_NIBBLE: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

#[macro_export]
macro_rules! byte_to_arr {
    ($b: expr) => {{
        let a = $b;
        [
            HEX_NIBBLE[(a & 0xf0) as usize >> 4],
            HEX_NIBBLE[(a & 0x0f) as usize],
        ]
    }};
}

/// Render `bytes` to the passed `Write` using their two-character hexadecimal repr
fn render_bytes(w: &mut impl Write, bytes: &[u8], count: usize) {
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if count > i {
            w.write(&byte_to_arr!(b)).unwrap();
        } else {
            w.write(b"  ").unwrap();
        }
        if i != bytes.len() - 1 {
            w.write(b" ").unwrap();
        }
        i += 1;
    }
}

/// Skip n bytes before a reader
fn skip_bytes(read: &mut impl Read, skip: usize) -> io::Result<usize> {
    if skip == 0 {
        Ok(0)
    } else {
        Ok(read.read(&mut vec![0u8; skip])?)
    }
}

/// Print out a number to `w` as 8-char hexadecimal
fn hex(w: &mut impl Write, num: usize) -> io::Result<()> {
    const MASK: usize = 0xf;

    macro_rules! f {
        ($i: literal) => {
            HEX_NIBBLE[(num & (MASK << $i * 4)) >> $i * 4]
        };
        ($($a: literal)+) => {
            [$(f!($a)),+]
        };
    }

    let a = f!(7 6 5 4 3 2 1 0);
    w.write_all(&a)?;

    Ok(())
}

fn print_canonical(
    out: &mut impl Write,
    mut read: impl Read,
    skip: usize,
    length: usize,
    squeeze: bool,
) -> io::Result<()> {
    const N: usize = 16;

    let mut bytes = [0u8; N];
    let mut i = skip_bytes(&mut read, skip)?;
    let end = if length == usize::MAX {
        None
    } else {
        Some(i + length)
    };

    let mut b = [0u8; N];
    let mut read_count = read.read(&mut bytes)?;

    // Whether the '*' char has been printed for the squeeze
    let mut printed_squeeze = false;
    // If we're actively suqeezing
    let mut squeezing = false;

    // `00000000  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|\n` = 79 B
    let mut line = Vec::with_capacity(79);
    while read_count != 0 {
        if let Some(end) = end {
            if read_count + i > end {
                read_count = end - i;
            }
        }
        if squeeze {
            b = *&bytes;
        }
        //b.copy_from_slice(&bytes);
        if !squeezing {
            line.clear();
            printed_squeeze = false;
            hex(&mut line, i)?;
            line.write(b"  ")?;
            render_bytes(&mut line, &bytes[..8], read_count);
            line.write(b"  ")?;
            render_bytes(
                &mut line,
                &bytes[8..],
                (read_count as isize - 8).max(0) as usize,
            );
            line.write(b"  |")?;

            render_ascii(&mut line, &mut bytes[..read_count])?;
            line.write(b"|\n")?;
            out.write(&line)?;
        }
        assert_eq!(line.capacity(), 79);
        i += read_count;
        if let Some(end) = end {
            if i >= end {
                break;
            }
        }

        read_count = read.read(&mut bytes)?;
        if read_count == N && bytes == b && squeeze {
            if !printed_squeeze {
                out.write(b"*\n")?;
                printed_squeeze = true;
            }
            squeezing = true;
        } else {
            squeezing = false;
        }
    }

    writeln!(out, "{:08x?}", i)?;
    Ok(())
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let mut stdout = BufWriter::new(stdout().lock());
    if let Some(file) = cli.file {
        let read = BufReader::new(fs::File::open(file)?);
        print_canonical(&mut stdout, read, cli.skip, cli.length, !cli.no_squeeze)?;
    } else {
        let read = BufReader::new(stdin().lock());
        print_canonical(&mut stdout, read, cli.skip, cli.length, !cli.no_squeeze)?;
    }

    Ok(())
}

fn main() {
    let res = run();
    if let Err(e) = res {
        eprintln!("{}", e);
    }
}
