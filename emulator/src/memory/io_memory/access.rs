pub trait LineGetter<T> {
    fn get_line(&self) -> T;
}

pub trait LineSender<T> {
    fn send_line(&self, line: &T);
}
