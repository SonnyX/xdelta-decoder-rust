use std::io;

static VCD_SELF: u8 = 0x00;
static VCD_HERE: u8 = 0x01;

#[derive(Debug)]
pub struct AddressCache {
    near: Vec<u64>,
    same: Vec<u64>,
    next_slot: usize,
}

impl AddressCache {
    pub fn new(near_sz: usize, same_sz: usize) -> AddressCache {
        AddressCache {
            near: vec![0; near_sz],
            same: vec![0; same_sz * 256],
            next_slot: 0,
        }
    }

    pub fn reset(&mut self) {
        for v in &mut self.near {
            *v = 0;
        }
        for v in &mut self.same {
            *v = 0;
        }
        self.next_slot = 0;
    }

    pub fn update(&mut self, addr: u64) {
        self.near[self.next_slot] = addr;
        self.next_slot = (self.next_slot + 1) % self.near.len();
        let same_len = self.same.len() as u64;
        self.same[(addr % same_len) as usize] = addr;
    }

    pub fn decode<'a>(&mut self, here: u64, mode: u8, input: &'a [u8] ) -> Result<(&'a [u8], u64), io::Error> {
        fn varint<'a>(input: &'a [u8]) -> Result<(&'a [u8], u64), io::Error> {
            let mut result : u64 = 0;
            let mut not_finished : bool = true;
            let mut counter = 0;
            while not_finished {
                if counter == 10 || counter == (input.len() + 1) {
                    return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "unable to get instruction address"));
                }
                let next_byte = input[counter];
                counter += 1;
                result = (result << 7) | (next_byte as u64 & 127);
                if (next_byte & 128) == 0 {
                    not_finished = false;
                }
            }
            return Ok((&input[counter..], result));
        }

        fn one<'a>(input: &'a [u8]) -> Result<(&'a [u8], u64), io::Error> {
            if input.len() > 0 {
                Ok((&input[1..], input[0] as u64))
            } else {
                Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "unable to get instruction address",
                ))
            }
        }

        let mut res: (&'a [u8], u64);
        if mode == VCD_SELF {
            res = varint(input)?;
        } else if mode == VCD_HERE {
            res = varint(input)?;
            res.1 = here - res.1;
        } else if mode >= 2 && (mode as usize) - 2 < self.near.len() {
            res = varint(input)?;
            res.1 = self.near[(mode as usize) - 2] + res.1;
        } else {
            res = one(input)?;
            let m = (mode as usize) - 2 - self.near.len();
            res.1 = self.same[m * 256 + res.1 as usize];
        }

        self.update(res.1);
        Ok(res)
    }

    pub fn encode(&mut self, addr: u64, here: u64) -> (u64, u8) {
        /* Attempt to find the address mode that yields the
         * smallest integer value for "d", the encoded address
         * value, thereby minimizing the encoded size of the
         * address. */
        let mut best = (addr, VCD_SELF);

        if here - addr < best.0 {
            best = (here - addr, VCD_HERE);
        }

        for (i, &near) in self.near.iter().enumerate() {
            if addr > near && addr - near < best.0 {
                best = (addr - near, (i as u8) + 2);
            }
        }

        let idx = (addr % (self.same.len() as u64)) as usize;
        if self.same[idx] == addr {
            best = ((idx % 256) as u64, (self.near.len() + 2 + idx / 256) as u8)
        }

        self.update(best.0);
        best
    }
}
