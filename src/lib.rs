extern crate lzma_sys;

mod vcdiff_header;
mod vcdiff_window;
mod vcdiff_address_cache;
mod vcdiff_code_table;
mod reader;

use vcdiff_header::Header;
use vcdiff_window::Window;

use lzma_sys::*;
use lzma_stream_wrapper::LzmaStreamWrapper;
use reader::Reader;

use std::fs::OpenOptions;
use std::path::Path;

pub fn decode_file<P: AsRef<Path>>(source_file_path: Option<P>, patch_file_path: P, target_file_path: P) {
  let mut source = match source_file_path {
    Some(path) => Some(OpenOptions::new().read(true).open(path).unwrap()),
    None => None
  };
  let patch = OpenOptions::new().read(true).open(patch_file_path).unwrap();
  let mut target = OpenOptions::new().read(true).write(true).create(true).open(target_file_path).unwrap();
  let mut bytes = Reader::with_capacity(200,patch);

  //read header
  let header = Header::new(&mut bytes);

  //initialize lzma-decompressor-streams
  let mut data_stream = LzmaStreamWrapper::new();
  data_stream.stream_decoder(std::u64::MAX, 0).unwrap();
  let mut instructions_stream = LzmaStreamWrapper::new();
  instructions_stream.stream_decoder(std::u64::MAX, 0).unwrap();
  let mut addresses_stream = LzmaStreamWrapper::new();
  addresses_stream.stream_decoder(std::u64::MAX, 0).unwrap();

  //read windows
  while bytes.peek().is_some() {
    let mut window = Window::new(&mut bytes);

    if header.secondary_compressor_id == Some(2) {
      //decompress lzma2
      if window.delta_indicator % 2 >= 1 {
        //decompress data
        let size = decode_base7_int(&mut window.data[0..10].iter());
        let mut decoded_data : Vec<u8> = Vec::with_capacity(size.result.unwrap() as usize);
        decoded_data.resize(size.result.unwrap() as usize,0);
        let result = data_stream.code(&mut window.data[size.bytes_read..], &mut decoded_data, lzma_action::LzmaRun);
        if result.ret.is_ok() {
          window.data = decoded_data;
          window.data_length = size.result.unwrap();
        }
      }
      if window.delta_indicator % 4 >= 2 {
        //decompress instructions
        let size = decode_base7_int(&mut window.instructions[0..10].iter());
        let mut decoded_instructions : Vec<u8> = Vec::with_capacity(size.result.unwrap() as usize);
        decoded_instructions.resize(size.result.unwrap() as usize,0);
        let result = instructions_stream.code(&mut window.instructions[size.bytes_read..], &mut decoded_instructions, lzma_action::LzmaRun);
        if result.ret.is_ok() {
          window.instructions = decoded_instructions;
          window.instructions_length = size.result.unwrap();
        }
      }
      if window.delta_indicator % 8 >= 4 {
        //decompress addresses
        let size = decode_base7_int(&mut window.addresses[0..10].iter());
        let mut decoded_addresses : Vec<u8> = Vec::with_capacity(size.result.unwrap() as usize);
        decoded_addresses.resize(size.result.unwrap() as usize,0);
        let result = addresses_stream.code(&mut window.addresses[size.bytes_read..], &mut decoded_addresses, lzma_action::LzmaRun);
        if result.ret.is_ok() {
          window.addresses = decoded_addresses;
          window.addresses_length = size.result.unwrap();
        }
      }
      window.delta_indicator = 0;
    }
    window.decode_window(&mut source, &mut target).unwrap();
  }
}

#[derive(Debug)]
pub struct DecodeResult {
  result: Option<u64>,
  bytes_read: usize,
}

pub fn decode_base7_int(bytes: &mut std::slice::Iter<'_, u8>) -> DecodeResult {
  let mut result : u64 = 0;
  let mut not_finished : bool = true;
  let mut counter = 0;
  while not_finished {
    if counter == 10 {
      return DecodeResult { result: None, bytes_read: counter };
    }
    counter += 1;
    let next_byte : u64 = bytes.next().unwrap().clone() as u64;
    result = (result << 7) | (next_byte & 127);
    if (next_byte & 128) == 0 {
      not_finished = false;
    }
  }
  return DecodeResult { result: Some(result), bytes_read: counter };
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn window_size() {
    decode_file(None, "/home/sonny/git/xdelta-decoder-rust/D1DECD86ECAFC8A8389C7DE49DC27DEA429C6C81E519CEB5815844C71BB8A83A", "/home/sonny/git/xdelta-decoder-rust/lol.map");
    assert!(true);
  }
}
