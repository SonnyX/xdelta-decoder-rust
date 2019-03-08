use vcdiff_address_cache::AddressCache;
use vcdiff_code_table::{InstructionType,Instruction,CodeTable};
use std::io::{Read,Write,Seek};
use reader::Reader;

pub struct Window {
  window_indicator: u8, //VCD_SOURCE, VCD_TARGET, VCD_ADLER32
  source_segment: Option<(u64,u64)>, //unimplemented behavior
  delta_encoding_length: u64, // size/length of the entire struct
  pub target_window_length: u64, // size of ??
  pub delta_indicator: u8,
  pub data_length: u64,
  pub instructions_length: u64,
  pub addresses_length: u64,
  adler32_checksum: Option<[u8;4]>,
  pub data: Vec<u8>,
  pub instructions: Vec<u8>,
  pub addresses: Vec<u8>,
}

impl std::fmt::Debug for Window {
  fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
    fmt.debug_struct("Window")
     .field("window_indicator", &self.window_indicator)
     .field("source_segment", &self.source_segment)
     .field("delta_encoding_length", &self.delta_encoding_length)
     .field("target_window_length", &self.target_window_length)
     .field("delta_indicator", &self.delta_indicator)
     .field("data_length", &self.data_length)
     .field("instructions_length", &self.instructions_length)
     .field("addresses_length", &self.addresses_length)
     .field("d.i.a_length", &(&self.data_length + &self.instructions_length + &self.addresses_length))
     .finish()
  }
}

impl Window {
  /**
  * Creates a new Window instance and uses an iterator to fill it with the data of a vcdiff
  */
  pub fn new(bytes: &mut Reader) -> Window {
    let mut window = Window {
      window_indicator: bytes.next().unwrap(), //1 byte
      source_segment: None,  //up to 20 bytes
      delta_encoding_length: 0, //up to 10 bytes
      target_window_length: 0, //up to 10 bytes
      delta_indicator: 0, //one byte
      data_length: 0,  //up to 10 bytes
      instructions_length: 0, //up to 10 bytes
      addresses_length: 0, //up to 10 bytes
      adler32_checksum: None,  //4 bytes
      data: Vec::new(),  
      instructions: Vec::new(),
      addresses: Vec::new(),
    };
    if window.window_indicator % 2 >= 1 || window.window_indicator % 4 >= 2 { //VCD_SOURCE || VCD_TARGET
      window.source_segment = Some((bytes.decode_base7_int().result.unwrap(), bytes.decode_base7_int().result.unwrap()));
    }
    window.delta_encoding_length = bytes.decode_base7_int().result.unwrap();
    window.target_window_length = bytes.decode_base7_int().result.unwrap();
    window.delta_indicator = bytes.next().unwrap();
    window.data_length = bytes.decode_base7_int().result.unwrap();
    window.instructions_length = bytes.decode_base7_int().result.unwrap();
    window.addresses_length = bytes.decode_base7_int().result.unwrap();
    if window.window_indicator % 8 >= 4 { //VCD_ADLER32
      window.adler32_checksum = Some([bytes.next().unwrap(),
                          bytes.next().unwrap(),
                          bytes.next().unwrap(),
                          bytes.next().unwrap()]);
    }

    // Data bytes
    bytes.seek(std::io::SeekFrom::Current(0)).unwrap();
    window.data = Vec::with_capacity(window.data_length as usize);
    window.data.resize(window.data_length as usize, 0);
    bytes.read(&mut window.data).unwrap();

    // Instructions bytes
    window.instructions = Vec::with_capacity(window.instructions_length as usize);
    window.instructions.resize(window.instructions_length as usize, 0);
    bytes.read(&mut window.instructions).unwrap();

    // Addresses bytes
    window.addresses = Vec::with_capacity(window.addresses_length as usize);
    window.addresses.resize(window.addresses_length as usize, 0);
    bytes.read(&mut window.addresses).unwrap();

    //return window
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
