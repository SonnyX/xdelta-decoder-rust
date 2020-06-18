use std::convert::From;
use std::result::Result;
use std::io::Error as IoError;

use super::{lzma_ret};


/// An error produced by an operation on LZMA data
#[derive(Debug)]
pub enum LzmaError {
	/// Failed Memory Allocation
    Mem,
	/// Memory limit would be violated
	MemLimit,
	/// XZ magic bytes weren't found
	Format,
	/// Unsupported compression options
	Options,
	/// Corrupt data
	Data,
	/// Data looks truncated
	Buf,
	/// std::io::Error
	Io(IoError),
	/// An unknown error
	Other,
}

impl std::fmt::Display for LzmaError {
	#[inline(always)]
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let details = match *self {
			LzmaError::Mem => "Memory allocation failed",
			LzmaError::MemLimit => "Memory limit would be violated",
			LzmaError::Format => "XZ magic bytes were not found",
			LzmaError::Options => "Unsupported compression options",
			LzmaError::Data => "Corrupt data",
			LzmaError::Buf => "Data look like it was truncated or possibly corrupt",
			LzmaError::Io(..) => "IO error",
			LzmaError::Other => "Unknown error",
		};
		write!(f,"{}", details)
	}
  }

impl From<IoError> for LzmaError {
	fn from(err: IoError) -> LzmaError {
		LzmaError::Io(err)
	}
}


/* Return values from liblzma are converted into this for easier handling */
pub type LzmaLibResult = Result<lzma_ret, LzmaError>;

impl From<lzma_ret> for LzmaLibResult {
	fn from(ret: lzma_ret) -> LzmaLibResult {
		match ret {
			lzma_ret::LzmaOk => Ok(ret),
			lzma_ret::LzmaStreamEnd => Ok(ret),
			lzma_ret::LzmaNoCheck => Ok(ret),
			lzma_ret::LzmaUnsupportedCheck => Ok(ret), // NOTE: This is an error in some cases.  Not sure how to handle properly.
			lzma_ret::LzmaGetCheck => Ok(ret),
			lzma_ret::LzmaMemError => Err(LzmaError::Mem),
			lzma_ret::LzmaMemlimitError => Err(LzmaError::MemLimit),
			lzma_ret::LzmaFormatError => Err(LzmaError::Format),
			lzma_ret::LzmaOptionsError => Err(LzmaError::Options),
			lzma_ret::LzmaDataError => Err(LzmaError::Data),
			lzma_ret::LzmaBufError => Err(LzmaError::Buf),
			_ => Err(LzmaError::Other),
		}
	}
}
