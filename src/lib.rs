mod interface;
mod packet;

pub mod setting;
pub mod result;
pub mod blocking;

#[cfg(feature = "async")]
#[cfg(not(target_os="windows"))]
pub mod async_io;

#[cfg(feature = "service")]
pub mod service;

#[cfg(feature = "os")]
pub mod os;
