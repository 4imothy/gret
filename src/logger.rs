use lazy_static::lazy_static;
use std::fs::File as FSFile;
use std::sync::Mutex;

lazy_static! {
    pub static ref LOG: Mutex<FSFile> = Mutex::new(FSFile::create("log").unwrap());
}

#[allow(unused_macros)]
macro_rules! log {
    ($($arg:tt)*) => {
        writeln!($crate::logger::LOG.lock().unwrap(), $($arg)*).unwrap();
    };
}

#[allow(unused_imports)]
pub(crate) use log;
