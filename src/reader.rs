use std::io::prelude::*;

use std::cmp;
use std::fmt;
use std::io::{self, SeekFrom}; 

pub struct Reader {
    inner: std::fs::File,
    buf: Box<[u8]>,
    pos: usize,
    cap: usize,
}

impl Reader {
    pub fn with_capacity(cap: usize, inner: std::fs::File) -> Reader {
        unsafe {
            let mut buffer = Vec::with_capacity(cap);
            buffer.set_len(cap);
            //inner.read(&mut buffer);
            Reader {
                inner,
                buf: buffer.into_boxed_slice(),
                pos: 0,
                cap: 0,
            }
        }
    }

    pub fn peek(&mut self) -> Option<u8>{
        if self.pos == self.cap && 1 >= self.buf.len() {
            return None;
        }
        let mut next = None;
        {
            match self.fill_buf() {
                Ok(array) => {
                    if array.len() != 0 {
                      next = Some(array[0]);
                    }
                },
                Err(_e) => {}
            }
        }
        //println!("{:?}", next);
        return next;
    }
    pub fn next(&mut self) -> Option<u8>{
        if self.pos == self.cap && 1 >= self.buf.len() {
            return None;
        }
        let mut next = None;
        {
            match self.fill_buf() {
                Ok(array) => {
                    next = Some(array[0]);
                },
                Err(_e) => {}
            }
        }
        if next.is_some() {
            self.consume(1);
        }
        //println!("{:?}", next);
        return next;
    }

    pub fn decode_base7_int(&mut self) -> DecodeResult {
      let mut result : u64 = 0;
      let mut not_finished : bool = true;
      let mut counter = 0;
      while not_finished {
        if counter == 10 {
          return DecodeResult { result: None, bytes_read: counter };
        }
        counter += 1;
        let next_byte = self.next().unwrap();
        result = (result << 7) | (next_byte as u64 & 127);
        if (next_byte & 128) == 0 {
          not_finished = false;
        }
      }
      return DecodeResult { result: Some(result), bytes_read: counter };
    }
}


impl Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // If we don't have any buffered data and we're doing a massive read
        // (larger than our internal buffer), bypass our internal buffer
        // entirely.
        if self.pos == self.cap && buf.len() >= self.buf.len() {
            return self.inner.read(buf);
        }
        let nread = {
            let mut rem = self.fill_buf()?;
            rem.read(buf)?
        };
        self.consume(nread);
        Ok(nread)
    }
}

impl BufRead for Reader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // If we've reached the end of our internal buffer then we need to fetch
        // some more data from the underlying reader.
        // Branch using `>=` instead of the more correct `==`
        // to tell the compiler that the pos..cap slice is always valid.
        if self.pos >= self.cap {
            debug_assert!(self.pos == self.cap);
            self.cap = self.inner.read(&mut self.buf)?;
            self.pos = 0;
        }
        Ok(&self.buf[self.pos..self.cap])
    }

    fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.cap);
    }
}

impl fmt::Debug for Reader {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Reader")
            .field("reader", &self.inner)
            .field("buffer", &format_args!("{}/{}", self.cap - self.pos, self.buf.len()))
            .finish()
    }
}

impl Seek for Reader {
    /// Seek to an offset, in bytes, in the underlying reader.
    ///
    /// The position used for seeking with `SeekFrom::Current(_)` is the
    /// position the underlying reader would be at if the `Reader` had no
    /// internal buffer.
    ///
    /// Seeking always discards the internal buffer, even if the seek position
    /// would otherwise fall within it. This guarantees that calling
    /// `.into_inner()` immediately after a seek yields the underlying reader
    /// at the same position.
    ///
    /// To seek without discarding the internal buffer, use [`Reader::seek_relative`].
    ///
    /// See [`std::io::Seek`] for more details.
    ///
    /// Note: In the edge case where you're seeking with `SeekFrom::Current(n)`
    /// where `n` minus the internal buffer length overflows an `i64`, two
    /// seeks will be performed instead of one. If the second seek returns
    /// `Err`, the underlying reader will be left at the same position it would
    /// have if you called `seek` with `SeekFrom::Current(0)`.
    ///
    /// [`Reader::seek_relative`]: struct.Reader.html#method.seek_relative
    /// [`std::io::Seek`]: trait.Seek.html
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let result: u64;
        if let SeekFrom::Current(n) = pos {
            let remainder = (self.cap - self.pos) as i64;
            // it should be safe to assume that remainder fits within an i64 as the alternative
            // means we managed to allocate 8 exbibytes and that's absurd.
            // But it's not out of the realm of possibility for some weird underlying reader to
            // support seeking by i64::min_value() so we need to handle underflow when subtracting
            // remainder.
            if let Some(offset) = n.checked_sub(remainder) {
                result = self.inner.seek(SeekFrom::Current(offset))?;
            } else {
                // seek backwards by our remainder, and then by the offset
                self.inner.seek(SeekFrom::Current(-remainder))?;
                self.pos = self.cap; // empty the buffer
                result = self.inner.seek(SeekFrom::Current(n))?;
            }
        } else {
            // Seeking with Start/End doesn't care about our buffer length.
            result = self.inner.seek(pos)?;
        }
        self.pos = self.cap; // empty the buffer
        Ok(result)
    }
}

#[derive(Debug)]
pub struct DecodeResult {
  pub result: Option<u64>,
  pub bytes_read: usize,
}
