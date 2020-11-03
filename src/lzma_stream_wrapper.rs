//! Wraps the underlying FFI struct `lzma_stream` to provide various safety guarantees, like the Send trait.

use super::{lzma_end, lzma_code, lzma_auto_decoder, lzma_stream, lzma_ret};
use lzma_error::{LzmaError, LzmaLibResult};
use lzma_action::LzmaAction;
use std::ptr;
use std::ops::Drop;
use std::mem;

pub struct LzmaStreamWrapper {
	stream: lzma_stream,
}

pub struct LzmaCodeResult {
	/// The return value of lzma_code
	pub ret: Result<lzma_ret, LzmaError>,
	/// The number of bytes read from input
	pub bytes_read: usize,
	/// The number of bytes written to output
	pub bytes_written: usize,
}


// I believe liblzma is at least Send thread safe, though using it like that will result in
// malloc being called in one thread and free being called in another.  That's usually safe,
// but depends on how liblzma was compiled.
unsafe impl Send for LzmaStreamWrapper {}


impl LzmaStreamWrapper {
	pub fn new() -> LzmaStreamWrapper {
		LzmaStreamWrapper {
			stream: unsafe { lzma_stream::from(mem::zeroed()) },
		}
	}

	pub fn stream_decoder(&mut self, memlimit: u64, flags: u32) -> Result<(), LzmaError> {
		let lzma_ret = unsafe { lzma_auto_decoder(&mut self.stream, memlimit, flags) };
		LzmaLibResult::from(lzma_ret).map(|_| ())
	}

	/// Pointers to input and output are given to liblzma during execution of this function,
	/// but they are removed before returning.  So that should keep everything safe.
	pub fn code(&mut self, input: &[u8], output: &mut [u8], action: LzmaAction) -> LzmaCodeResult {
		// Prepare lzma_stream
		self.stream.next_in = input.as_ptr();
		self.stream.avail_in = input.len();
		self.stream.next_out = output.as_mut_ptr();
		self.stream.avail_out = output.len();
		// Execute lzma_code and get results
		let mut ret = unsafe {
			LzmaLibResult::from(lzma_code(&mut self.stream, action.into()))
		};
    while ret.is_ok() && self.stream.avail_in > 0 {
      ret = unsafe {
			  LzmaLibResult::from(lzma_code(&mut self.stream, action.into()))
		  };
    }
		let bytes_read = input.len() - self.stream.avail_in;
		let bytes_written = output.len() - self.stream.avail_out;
		// Clear pointers from lzma_stream
		self.stream.next_in = ptr::null();
		self.stream.avail_in = 0;
		self.stream.next_out = ptr::null_mut();
		self.stream.avail_out = 0;

		LzmaCodeResult {
			ret: ret,
			bytes_read: bytes_read,
			bytes_written: bytes_written,
		}
	}
}

// This makes sure to call lzma_end, which frees memory that liblzma has allocated internally
// Note: It appears to be safe to call lzma_end multiple times; so this Drop is safe
// even if the user has already called end.
impl Drop for LzmaStreamWrapper {
	fn drop(&mut self) {
		unsafe {
			lzma_end(&mut self.stream)
		}
	}
}
