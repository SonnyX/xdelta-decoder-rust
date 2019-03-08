use reader::Reader;
use std::io::{Seek,Read};

#[derive(Debug)]
pub struct CodeTable {
  near_cache_size: u8,
  same_cache_size: u8,
  compressed_data: Vec<u8>
}

#[derive(Debug)]
pub struct Header {
  pub header: [u8;4],
  pub hdr_indicator: u8, //Something like the version afaik.
  pub secondary_compressor_id: Option<u8>, // number 2 is LZMA2
  pub code_table_length: Option<u64>,
  pub code_table: Option<CodeTable>,
  pub appheader_size: Option<u64>,
  pub appheader: Vec<u8>
}

impl Header {
  pub fn new(bytes: &mut Reader) -> Header {
    let mut header = Header {
      header: [bytes.next().unwrap(),
               bytes.next().unwrap(),
               bytes.next().unwrap(),
               bytes.next().unwrap()],
      hdr_indicator: bytes.next().unwrap(),
      secondary_compressor_id: None,
      code_table_length: None,
      code_table: None,
      appheader_size: None,
      appheader: Vec::new(),
    };
    if header.hdr_indicator % 2 >= 1 { //VCD_SECONDARY
      header.secondary_compressor_id = Some(bytes.next().unwrap());
    }
    if header.hdr_indicator % 4 >= 2 { //VCD_CODETABLE
      header.code_table_length = bytes.decode_base7_int().result;
      let mut code_table = CodeTable{
                                 near_cache_size: bytes.next().unwrap(),
                                 same_cache_size: bytes.next().unwrap(),
                                 compressed_data: Vec::with_capacity(header.code_table_length.unwrap() as usize)
                               };
      bytes.seek(std::io::SeekFrom::Current(0)).unwrap();
      code_table.compressed_data.resize(header.code_table_length.unwrap() as usize,0);
      bytes.read(&mut code_table.compressed_data).unwrap();
      header.code_table = Some(code_table);
    }
    if header.hdr_indicator % 8 >= 4 { //VCD_APPHEADER
      header.appheader_size = bytes.decode_base7_int().result;
      bytes.seek(std::io::SeekFrom::Current(0)).unwrap();
      header.appheader = Vec::with_capacity(header.appheader_size.unwrap() as usize);
      header.appheader.resize(header.appheader_size.unwrap() as usize,0);
      bytes.read(&mut header.appheader).unwrap();
    }
    header
  }
}
