
#[derive(Debug)]
pub struct Window {
  window_indicator: u8, //VCD_SOURCE, VCD_TARGET, VCD_ADLER32
  source_segment_length: Option<u8>, //unimplemented behavior
  source_segment_position: Option<u8>, //unimplemented behavior
  delta_encoding_length: u64, // size/length of the entire struct
  target_window_length: u64, // size of ??
  pub delta_indicator: u8,
  pub data_length: u64,
  pub instructions_length: u64,
  pub addresses_length: u64,
  adler32_checksum: Option<[u8;4]>,
  pub data: Vec<u8>,
  pub instructions: Vec<u8>,
  pub addresses: Vec<u8>,
}

impl Window {
  /**
  * Creates a new Window instance and uses an iterator to fill it with the data of a vcdiff
  */
  pub fn new(bytes: &mut std::iter::Peekable<std::io::Bytes<std::fs::File>>) -> Window {
    let mut window = Window {
      window_indicator: bytes.next().unwrap().unwrap(),
      source_segment_length: None,
      source_segment_position: None,
      delta_encoding_length: 0,
      target_window_length: 0,
      delta_indicator: 0,
      data_length: 0,
      instructions_length: 0,
      addresses_length: 0,
      adler32_checksum: None,
      data: Vec::new(),
      instructions: Vec::new(),
      addresses: Vec::new(),
    };
    if window.window_indicator % 2 >= 1 || window.window_indicator % 4 >= 2 { //VCD_SOURCE || VCD_TARGET
      window.source_segment_length = Some(bytes.next().unwrap().unwrap());
      window.source_segment_position = Some(bytes.next().unwrap().unwrap());
    }
    window.delta_encoding_length = decode_base7_int(bytes).result.unwrap();
    window.target_window_length = decode_base7_int(bytes).result.unwrap();
    window.delta_indicator = bytes.next().unwrap().unwrap();
    window.data_length = decode_base7_int(bytes).result.unwrap();
    window.instructions_length = decode_base7_int(bytes).result.unwrap();
    window.addresses_length = decode_base7_int(bytes).result.unwrap();
    if window.window_indicator % 8 >= 4 { //VCD_ADLER32
      window.adler32_checksum = Some([bytes.next().unwrap().unwrap(),
                          bytes.next().unwrap().unwrap(),
                          bytes.next().unwrap().unwrap(),
                          bytes.next().unwrap().unwrap()]);
    }

    // Data bytes
    window.data.reserve(window.data_length as usize);
    for n in 0..(window.data_length as usize) {
      window.data.push(bytes.next().unwrap().unwrap());
    }

    // Instructions bytes
    window.instructions.reserve(window.instructions_length as usize);
    for n in 0..(window.instructions_length as usize) {
      window.instructions.push(bytes.next().unwrap().unwrap());
    }

    // Addresses bytes
    window.addresses.reserve(window.addresses_length as usize);
    for n in 0..(window.addresses_length as usize) {
      window.addresses.push(bytes.next().unwrap().unwrap());
    }
    window
  }
  
  
}

#[derive(Debug)]
pub struct DecodeResult {
  result: Option<u64>,
  bytes_read: usize,
}

pub fn decode_base7_int(bytes: &mut std::iter::Peekable<std::io::Bytes<std::fs::File>>) -> DecodeResult {
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
