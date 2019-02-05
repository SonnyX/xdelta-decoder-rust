use vcdiff_address_cache::AddressCache;
use vcdiff_code_table::{InstructionType,Instruction,CodeTable};
//use std::io;
use std::io::{Read,Write,Seek};

#[derive(Debug)]
pub struct Window {
  window_indicator: u8, //VCD_SOURCE, VCD_TARGET, VCD_ADLER32
  source_segment: Option<(u64,u64)>, //unimplemented behavior
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
      source_segment: None,
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
      window.source_segment = Some((decode_base7_int(bytes).result.unwrap(),decode_base7_int(bytes).result.unwrap()));
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

  pub fn decode_window(self, original: &mut Option<std::fs::File>, target: &mut std::fs::File) -> Result<(), std::io::Error> {
    let mut remaining_adds_runs = &self.data[..];
    let mut remaining_addresses = &self.addresses[..];
    let mut target_data = Vec::with_capacity(self.target_window_length as usize);
    //println!("Win indicator: {:02x}", self.window_indicator);
    //target_data.resize(self.target_window_length as usize, 0);
    //println!("target_window_length: {}", self.target_window_length);
    let mut address_cache = AddressCache::new(4,3);
    let window_header = &self;
    //println!("Well this is embarrasing: {:?}", &self);
    if window_header.delta_indicator > 0 {
      Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unsupported compression type"))?;
    }
    {
      let mut decode_instruction = | inst: Instruction, instructions: &[u8] | -> Result<usize, std::io::Error> {
        let mut size = inst.size as usize;
        let mut remaining_instructions = instructions;
        if size == 0 {
          let mut result : usize = 0;
          let mut not_finished : bool = true;
          let mut counter = 0;
          while not_finished {
            if counter == 10 || counter == (remaining_instructions.len()) {
              return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "unable to get instruction address"));
            }
            let next_byte = remaining_instructions[counter];
            counter += 1;
            result = (result << 7) | (next_byte as usize & 127);
            if (next_byte & 128) == 0 {
              not_finished = false;
            }
          }
					remaining_instructions = &remaining_instructions[counter..];
          size = result;
        }
        match inst.typ {
          InstructionType::Add => {
            if size == 0 { println!("Add: size = {}", size) };
            target_data.extend_from_slice(&remaining_adds_runs[0..size]);
            remaining_adds_runs = &remaining_adds_runs[size..];
          }
          InstructionType::Copy => {
            //println!("source_segment: {:?}", window_header.source_segment);
            let source_length = window_header.source_segment.map_or(0u64, |r| r.0);
            let (r, addr) = address_cache.decode((target_data.len() as u64) + source_length, inst.mode,remaining_addresses,)?;
            remaining_addresses = &r;
            let s = window_header.source_segment.and_then(|(sz, pos)| {
              if addr < sz {
                Some(pos)
              } else {
                None
              }
            });

            if let Some(pos) = s {
              let target_pos = target_data.len();
              target_data.resize(target_pos + size, 0u8);
              if window_header.window_indicator % 2 >= 1 { //VCD_SOURCE
                original.as_mut().unwrap().seek(std::io::SeekFrom::Start(pos + addr))?;
                original.as_mut().unwrap().read(&mut target_data[target_pos..target_pos + size])?;
                //println!("VCD_SOURCE: {:?}",&target_data[target_pos..target_pos + size]);
              } else {
                let current = target.seek(std::io::SeekFrom::Current(0))?;
                target.seek(std::io::SeekFrom::Start(pos + addr))?;
                target.read(&mut target_data[target_pos..target_pos + size])?;
                target.seek(std::io::SeekFrom::Start(current))?;
                println!("VCD_TARGET: {:?}",&target_data[target_pos..target_pos + size]);
              }
            } else {
              let target_pos = (addr - source_length) as usize;
              // probably quite slow...
              //println!("iterating over: {}..{}", target_pos, target_pos + size);
              for idx in target_pos..target_pos + size {
                let byte = target_data[idx];
                target_data.push(byte);
              }
            }
          }
          InstructionType::Run => {
            let byte = remaining_adds_runs[0];
            let pos = target_data.len();
            remaining_adds_runs = &remaining_adds_runs[1..];
            target_data.resize(pos + size, byte);
          }
        };
        Ok(instructions.len() - remaining_instructions.len())
      };
      let mut remaining_instructions = &self.instructions[..];
      let code_table = CodeTable::default();
      while let Some((&inst_index, r)) = remaining_instructions.split_first() {
        let e = code_table.entries[inst_index as usize];
        remaining_instructions = r;
        remaining_instructions = &remaining_instructions[decode_instruction(e.0, remaining_instructions)?..];
        if let Some(inst) = e.1 { //if e.1 exists unwrap it as the var "inst"
          remaining_instructions = &remaining_instructions[decode_instruction(inst, remaining_instructions)?..];
        }
      }
    }
    target.write(&target_data)?;
    Ok(())
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
