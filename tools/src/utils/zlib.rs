use crate::Result;
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use std::io::{Read, Write};

pub fn compress_bytes(data: &[u8]) -> Result<Vec<u8>> {
    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(data)?;
    Ok(encoder.finish()?)
}

pub fn decompress_bytes(data: &[u8]) -> Result<Vec<u8>> {
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();

    decoder.read_to_end(&mut decompressed)?;

    Ok(decompressed)
}

pub fn compress(data: impl AsRef<[u8]>) -> Result<Vec<u8>> {
    compress_bytes(data.as_ref())
}

pub fn decompress(data: impl AsRef<[u8]>) -> Result<Vec<u8>> {
    decompress_bytes(data.as_ref())
}
