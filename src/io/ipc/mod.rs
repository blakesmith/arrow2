//! APIs to read from and write to Arrow's IPC format.

mod compression;
mod convert;

pub use convert::fb_to_schema;
pub use arrow_format::ipc::Message::root_as_message;
pub mod read;
pub mod write;

const ARROW_MAGIC: [u8; 6] = [b'A', b'R', b'R', b'O', b'W', b'1'];
const CONTINUATION_MARKER: [u8; 4] = [0xff; 4];
