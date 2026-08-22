use std::io::stdin;

use emulator::memory::io_memory::access::{LineGetter, LineSender};


#[derive(Clone, Debug, Default)]
pub struct StdIOAccessor;

impl LineGetter<String> for StdIOAccessor {
    fn get_line(&self) -> String {
        let mut buf = Default::default();
        _ = stdin().read_line(&mut buf);
        buf
    }
}

impl LineSender<String> for StdIOAccessor {
    fn send_line(&self, line: &String) {
        print!("{line}");
    }
}
