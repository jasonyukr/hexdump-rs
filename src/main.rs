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
        *b = match b {
            b'"'..=b'}' | b' ' => *b,
            _ => INVALID_ASCII,
        };
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
fn render_bytes(w: &mut impl Write, bytes: &[u8], count: usize) -> io::Result<()> {
    for b in bytes.iter().take(count) {
        w.write(&byte_to_arr!(b))?;
        w.write(b" ")?;
    }

    if bytes.len() > count {
        w.write(&b"   ".repeat(bytes.len() - count))?;
    }

    Ok(())
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
    let end = (i + length).max(usize::MAX);
    let mut b = [0u8; N];
    let mut read_count = read.read(&mut bytes)?;

    enum SqueezeStatus {
        NotSqueezing,
        NotYetPrintedSqueeze,
        Squeezing,
    }

    let mut squeeze_status = SqueezeStatus::NotSqueezing;

    //  v---10---vv----------------------3N----------------------vv---------21--------v
    // `00000000  00 00 00 00 00 00 00 00  00 00 00 00 00 00 00 00  |................|\n`
    let mut line = Vec::with_capacity(10 + N * 3 + 21);

    while read_count != 0 {
        if read_count + i > end {
            read_count = end - i;
        }

        if squeeze {
            b = *&bytes;
        }

        if matches!(squeeze_status, SqueezeStatus::NotSqueezing) {
            line.clear();
            hex(&mut line, i)?;
            line.write(b"  ")?;
            render_bytes(&mut line, &bytes[..8], read_count)?;
            line.write(b" ")?;
            render_bytes(
                &mut line,
                &bytes[8..],
                if read_count < 8 { 0 } else { read_count },
            )?;
            line.write(b" |")?;

            render_ascii(&mut line, &mut bytes[..read_count])?;
            line.write(b"|\n")?;
            out.write(&line)?;
        }
        debug_assert_eq!(line.capacity(), 79);
        i += read_count;
        if i >= end {
            break;
        }

        read_count = read.read(&mut bytes)?;
        if read_count == N && bytes == b && squeeze {
            squeeze_status = match squeeze_status {
                SqueezeStatus::NotSqueezing => SqueezeStatus::NotYetPrintedSqueeze,
                SqueezeStatus::NotYetPrintedSqueeze => {
                    out.write(b"*\n")?;
                    SqueezeStatus::Squeezing
                }
                SqueezeStatus::Squeezing => squeeze_status,
            };
        } else {
            squeeze_status = SqueezeStatus::NotSqueezing;
        }
    }

    hex(out, i)?;
    out.write(&[b'\n'])?;
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
