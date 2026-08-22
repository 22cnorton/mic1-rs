use emulator::memory::io_memory::access::{LineGetter, LineSender};
use flume::{Receiver, Sender};

#[derive(Clone, Debug)]
pub struct ChannelLineAccessor {
    pub(crate) rx: Receiver<String>,
    pub(crate) tx: Sender<String>,
}

impl LineGetter<String> for ChannelLineAccessor {
    fn get_line(&self) -> String {
        match self.rx.recv() {
            Ok(s) => s,
            Err(_) => Default::default(),
        }
    }
}

impl LineSender<String> for ChannelLineAccessor {
    fn send_line(&self, line: &String) {
        _ = self.tx.send(line.to_string());
    }
}
