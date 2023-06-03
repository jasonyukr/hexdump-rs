use super::*;

use std::io::BufReader;

macro_rules! iter_to_string {
    ($it: expr) => {
        $it.iter().map(|b| *b as char).collect::<String>()
    };
}

#[test]
fn byte_hex() {
    for i in 0..u8::MAX {
        assert_eq!(iter_to_string!(byte_to_arr!(i)), format!("{:02x?}", i));
    }
}

#[test]
fn hex_fn() {
    for i in (0..u32::MAX).step_by(123456 /* Random number */) {
        let mut s = Vec::new();
        hex(&mut s, i as usize).unwrap();
        assert_eq!(iter_to_string!(s), format!("{:08x?}", i));
    }
}

#[test]
fn ascii() {
    macro_rules! test_ascii {
        ($arr: literal, $str: literal) => {{
            let mut v = Vec::new();
            let a = &mut $arr.clone();
            render_ascii(&mut v, a).unwrap();
            assert_eq!(iter_to_string!(v), $str);
        }};
    }

    test_ascii!(b"hello", "hello");
    test_ascii!(b"world", "world");
    test_ascii!(b"G'day", "G'day");
    test_ascii!(b"     ", "     ");
    test_ascii!(b"\n", ".");
    test_ascii!(b"\r", ".");
    test_ascii!(b"\x1b", ".");
}

#[test]
fn full() {
    let b = b"abcdefghijklmnop";
    assert_eq!(b.len(), 16); // Sanity check

    let mut s = Vec::new();
    print_canonical(&mut s, BufReader::new(&b[..]), 0, usize::MAX, true).unwrap();

    dbg!(String::from_utf8(s.clone())).unwrap();

    let target = b"00000000  61 62 63 64 65 66 67 68  69 6a 6b 6c 6d 6e 6f 70  |abcdefghijklmnop|
00000010
";
    assert_eq!(s, target);
}

#[test]
fn squeeze() {
    let b = "abcdefghijklmnop".repeat(7);
    assert_eq!(b.len(), 16 * 7); // Sanity check

    let mut s = Vec::new();
    print_canonical(&mut s, BufReader::new(b.as_bytes()), 0, usize::MAX, true).unwrap();

    dbg!(String::from_utf8(s.clone())).unwrap();

    let target = b"00000000  61 62 63 64 65 66 67 68  69 6a 6b 6c 6d 6e 6f 70  |abcdefghijklmnop|
*
00000070
";
    assert_eq!(s, target);
}
