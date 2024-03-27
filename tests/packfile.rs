mod common;

use common::*;

const PACKFILE: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/pack"));

// we aren't handling delta objects
#[derive(Debug, Clone, Copy)]
enum PackfileObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
}

fn parse_type_length<T: Read>(data: &mut T) -> (PackfileObjectType, u64) {
    use PackfileObjectType::*;
    let mut vec = Vec::new();
    let mut reader = data.bytes();
    loop {
        let byte = reader.next().unwrap().unwrap();
        vec.push(byte);
        if byte >> 7 != 0 {
            continue;
        } else {
            break;
        }
    }
    let object_type = match vec[0] >> 4 & 0b111 {
        1 => Commit,
        2 => Tree,
        3 => Blob,
        4 => Tag,
        _ => todo!(),
    };
    let object_length = (vec[1..]
        .iter()
        .copied()
        .rev()
        .fold(0u64, |acc, byte| (acc << 7) | (byte & 0b111_1111) as u64)
        << 4)
        | (vec[0] & 0xf) as u64;
    (object_type, object_length)
}

fn parse_header<T: Read>(reader: T) -> Vec<u8> {
    reader.bytes().map(|byte| byte.unwrap()).take(12).collect()
}

fn parse_number(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .copied()
        .fold(0, |acc, byte| acc << 8 | byte as u64)
}

macro_rules! assert_packfile {
    ($reader:expr, $decompressfn:expr, [version => $version:pat, count => $count:pat], $([$p:pat, $length:pat]),+) => {
        {
            let header = parse_header(&mut $reader);
            assert!(matches!(
                (&header[..4], parse_number(&header[4..8]), parse_number(&header[8..12])),
                (b"PACK", $version, $count)
            ));
            $(
                let (object_type, object_length) = parse_type_length(&mut $reader);
                let object_data = $decompressfn(&mut $reader, true);
                assert!(matches!(
                    (object_type, object_length, object_data.len()),
                    ($p, $length, $length)
                ));
            )+
            assert_eq!($reader.bytes().count(), 20);
        }
    };
}

#[test]
fn packfile() {
    use PackfileObjectType::*;
    let mut packfile = PACKFILE;
    assert_packfile!(
        &mut packfile,
        decompress,
        [version => 2, count => 0x9],
        [Commit, 282],
        [Tree, 30],
        [Tree, 103],
        [Tree, 32],
        [Blob, 2529],
        [Tree, 77],
        [Blob, 21793],
        [Blob, 49],
        [Blob, 12526]
    );
}
#[test]
#[should_panic]
fn packfile_flate2() {
    use PackfileObjectType::*;
    let mut packfile = PACKFILE;
    assert_packfile!(
        &mut packfile,
        decompress_flate2,
        [version => 2, count => 0x9],
        [Commit, 282],
        [Tree, 30],
        [Tree, 103],
        [Tree, 32],
        [Blob, 2529],
        [Tree, 77],
        [Blob, 21793],
        [Blob, 49],
        [Blob, 12526]
    );
}
