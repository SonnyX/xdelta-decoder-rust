mod lzma_error;
mod lzma_sys;
mod lzma_stream_wrapper;

mod vcdiff_header;
mod vcdiff_window;

use vcdiff_header::Header;
use vcdiff_window::Window;

use lzma_sys::*;
use lzma_stream_wrapper::LzmaStreamWrapper;

use std::io::Read;
use std::fs::OpenOptions;

fn main() {
  //let patch = OpenOptions::new().read(true).open("/home/sonny/git/RenegadeX-patcher-lib/RenegadeX/patcher/12A3F14FC43BB76C8E58DA0B0FE493C7DB360C1F029281AD6179F4E08C4D1A9E").unwrap();
  let patch = OpenOptions::new().read(true).open("/home/sonny/git/RenegadeX-patcher-lib/RenegadeX/patcher/07029D635639910C405FD5D9B89A7D77AFFE5675C7AB05848813900E146E9646").unwrap();
  let mut bytes = patch.bytes();
  let header = Header::new(&mut bytes);
  let mut window = Window::new(&mut bytes);

  let mut decoded_data : Vec<u8> = Vec::with_capacity(window.decoded_data_length.unwrap() as usize);
  decoded_data.resize(window.decoded_data_length.unwrap() as usize,0);
  let mut stream = LzmaStreamWrapper::new();
  stream.stream_decoder(std::u64::MAX, 0);
  let result = stream.code(&mut window.data, &mut decoded_data, lzma_action::LzmaRun);

  println!("{:?}",decoded_data);
  println!("{:?}",result);
}
