use std::convert::From;
use std::result::Result;
use std::io::Error as IoError;

use super::lzma_ret;


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
pub struct LzmaLibResult;

impl LzmaLibResult {
	pub fn from(ret: lzma_ret) -> Result<lzma_ret, LzmaError> {
		match ret {
			0 => Ok(ret), // Ok
			1 => Ok(ret), // Stream end
			2 => Ok(ret), // No Check
			3 => Ok(ret), // Unsupported Check, NOTE: This is an error in some cases.  Not sure how to handle properly.
			4 => Ok(ret), // Get Check
			5 => Err(LzmaError::Mem), // Mem Error
			6 => Err(LzmaError::MemLimit), // Mem limit Error
			7 => Err(LzmaError::Format), // Format Error
			8 => Err(LzmaError::Options), //Options Error
			9 => Err(LzmaError::Data), // Data Error
			10 => Err(LzmaError::Buf), // Buf Error
			_ => Err(LzmaError::Other), // Prog Error
		}
	}
}