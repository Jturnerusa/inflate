#![allow(dead_code)]

pub use std::io::prelude::*;

pub fn compress(data: &[u8]) -> Vec<u8> {
    let mut compress = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
    compress.write_all(data).unwrap();
    compress.finish().unwrap()
}

pub fn decompress<T: BufRead>(mut data: T, header: bool) -> Vec<u8> {
    let deflater = inflate::Decompress::new(&mut data, header);
    deflater.bytes().map(|b| b.unwrap()).collect()
}

pub fn decompress_flate2<T: Read>(data: T, _: bool) -> Vec<u8> {
    let deflater = flate2::read::ZlibDecoder::new(data);
    deflater.bytes().map(|b| b.unwrap()).collect()
}
