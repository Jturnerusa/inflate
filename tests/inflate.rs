mod common;

use common::*;

#[test]
fn roundtrip() {
    let data = b"hello world";
    assert_eq!(decompress(compress(data).as_slice(), true), data);
}
