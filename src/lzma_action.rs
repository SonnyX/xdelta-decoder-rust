#[repr(C)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum LzmaAction {
	LzmaRun           = 0,
	LzmaSyncFlush     = 1,
	LzmaFullFlush     = 2,
	LzmaFullBarrier   = 4,
	LzmaFinish        = 3,
}

impl Into<u32> for LzmaAction {
  fn into(self) -> u32 {
    self as u32
  }
}