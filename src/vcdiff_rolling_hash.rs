use std::io::Read;

/// a prime number
static A_PRIME: u32 = 257;

/// all math is done modulo n
/// n is a power of 2 to allow efficient modulus
/// n <= 2^32/a
/// n = (1 << 23)
static N_EFFICIENT: u32 = (1 << 23) - 1;

pub struct RollingHash {
    window_size: usize,
    /// for all b in 0..256, remove_table[b] = (- b * base^(window_size - 1)) % modulus
    remove_table: [u32; 256],
}

impl RollingHash {
    pub fn new(window_size: usize) -> RollingHash {
        let mut remove_table = [0u32; 256];

        let mut m = 1u32; // base^(window_size - 1) % modulus
        for _ in 0..window_size - 1 {
            m = (m * A_PRIME) & N_EFFICIENT;
        }

        for b in 0..256 {
            remove_table[b] = ((b as u32) * m).wrapping_neg() & N_EFFICIENT;
        }

        RollingHash {
            remove_table: remove_table,
            window_size,
        }
    }

    pub fn window_size(&self) -> usize {
        self.window_size
    }

    pub fn hash(&self, data: &[u8]) -> u32 {
        assert!(data.len() == self.window_size);

        let mut hash = 0u32;
        for &byte in data {
            hash = self.push_back(hash, byte);
        }

        hash
    }

    pub fn shift(&self, old_hash: u32, first_byte: u8, last_byte: u8) -> u32 {
        self.push_back(self.pop_front(old_hash, first_byte), last_byte)
    }

    fn push_back(&self, old_hash: u32, last_byte: u8) -> u32 {
        ((old_hash * A_PRIME) + (last_byte as u32)) & N_EFFICIENT
    }

    fn pop_front(&self, old_hash: u32, first_byte: u8) -> u32 {
        (old_hash + self.remove_table[first_byte as usize]) & N_EFFICIENT
    }
}

#[cfg(test)]
mod tests {
    use super::RollingHash;

    #[test]
    fn window_4() {
        let r = RollingHash::new(4);
        assert_eq!(r.hash(&[0, 0, 0, 0]), 0);
        assert_eq!(r.hash(&[0, 0, 0, 1]), 1);
        assert_eq!(r.hash(&[10, 20, 30, 40]), 3_302_500);
        assert_eq!(r.hash(&[255, 255, 255, 255]), 130_556);

        let mut h = r.hash(&[1, 54, 98, 165]);
        assert_eq!(h, 3_789_374);

        h = r.shift(h, 1, 241);
        assert_eq!(h, 396_590);

        h = r.shift(h, 54, 21);
        assert_eq!(h, 5_137_165);
    }
}
