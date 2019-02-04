mod lzma_error;
mod lzma_sys;
mod lzma_stream_wrapper;

mod vcdiff_header;
mod vcdiff_window;
mod vcdiff_address_cache;
mod vcdiff_code_table;

use vcdiff_header::Header;
use vcdiff_window::Window;

use lzma_sys::*;
use lzma_stream_wrapper::LzmaStreamWrapper;

use std::io::Read;
use std::fs::OpenOptions;

fn main() {
  let mut source = OpenOptions::new().read(true).open("/home/sonny/git/CNC-Walls-patched.udk").unwrap();
  let patch = OpenOptions::new().read(true).open("/home/sonny/git/B40C03E87FC7A66FF7B29AC3026E6EED636A8B6F84185A536FD637481E18F01F_from_996FB03AB19139EDF45E117AC3C57152DEEC16444654E03B447F35A92B104FAB").unwrap();
  //let patch = OpenOptions::new().read(true).open("/home/sonny/git/RenegadeX-patcher-lib/RenegadeX/patcher/07029D635639910C405FD5D9B89A7D77AFFE5675C7AB05848813900E146E9646").unwrap();
  let mut target_file = OpenOptions::new().read(true).write(true).create(true).open("/home/sonny/git/CNC-Walls-patched.udk").unwrap();
  let mut bytes = patch.try_clone().unwrap().bytes().peekable();

  //read header
  let header = Header::new(&mut bytes);

  //initialize lzma-decompressor-streams
  let mut data_stream = LzmaStreamWrapper::new();
  data_stream.stream_decoder(std::u64::MAX, 0);
  let mut instructions_stream = LzmaStreamWrapper::new();
  instructions_stream.stream_decoder(std::u64::MAX, 0);
  let mut addresses_stream = LzmaStreamWrapper::new();
  addresses_stream.stream_decoder(std::u64::MAX, 0);

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
    window.decode_window(Some(&mut source), &mut target_file);
    //println!("{:?}", window);
    //println!("{:?}",decoded_data);
    //println!("{:?}",result);
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
