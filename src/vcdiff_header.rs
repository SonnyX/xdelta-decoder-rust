#[derive(Debug)]
pub struct CodeTable {
  near_cache_size: u8,
  same_cache_size: u8,
  compressed_data: Vec<u8>
}

#[derive(Debug)]
pub struct Header {
  header: [u8;4],
  hdr_indicator: u8,
  secondary_compressor_id: Option<u8>,
  code_table_length: Option<u64>,
  code_table: Option<CodeTable>,
  appheader_size: Option<u8>,
  appheader: Vec<u8>
}

impl Header {
  pub fn new(bytes: &mut std::io::Bytes<std::fs::File>) -> Header {
    let mut header = Header {
      header: [bytes.next().unwrap().unwrap(),
               bytes.next().unwrap().unwrap(),
               bytes.next().unwrap().unwrap(),
               bytes.next().unwrap().unwrap()],
      hdr_indicator: bytes.next().unwrap().unwrap(),
      secondary_compressor_id: None,
      code_table_length: None,
      code_table: None,
      appheader_size: None,
      appheader: Vec::new(),
    };
    if header.hdr_indicator % 2 >= 1 { //VCD_SECONDARY
      header.secondary_compressor_id = Some(bytes.next().unwrap().unwrap());
    }
    if header.hdr_indicator % 4 >= 2 { //VCD_CODETABLE
      header.code_table_length = decode_base7_int(bytes).result;
      let mut code_table = CodeTable{
                                 near_cache_size: bytes.next().unwrap().unwrap(),
                                 same_cache_size: bytes.next().unwrap().unwrap(),
                                 compressed_data: Vec::new()
                               };
      for i in 0..header.code_table_length.unwrap() {
        code_table.compressed_data.push(bytes.next().unwrap().unwrap());
      }
      header.code_table = Some(code_table);
    }
    if header.hdr_indicator % 8 >= 4 { //VCD_APPHEADER
      header.appheader_size = Some(bytes.next().unwrap().unwrap());
      header.appheader = Vec::with_capacity(header.appheader_size.unwrap() as usize);
    }
    header
  }
}

#[derive(Debug)]
pub struct DecodeResult {
  result: Option<u64>,
  bytes_read: usize,
}

fn decode_base7_int(bytes: &mut std::io::Bytes<std::fs::File>) -> DecodeResult {
  let mut result : u64 = 0;
  let mut not_finished : bool = true;
  let mut counter = 0;
  while not_finished {
    if counter == 10 {
      return DecodeResult { result: None, bytes_read: counter };
    }
    counter += 1;
    let next_byte = bytes.next().unwrap().unwrap();
    result = (result << 7) | (next_byte as u64 & 127);
    if (next_byte & 128) == 0 {
      not_finished = false;
    }
  }
  return DecodeResult { result: Some(result), bytes_read: counter };
}

