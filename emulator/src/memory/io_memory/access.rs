pub trait LineGetter<T> {
    fn get_line(&self) -> T;
}

pub trait LineSender<T> {
    fn send_line(&self, line: &T);
}

pub trait LineAccessor<T>: LineSender<T> + LineGetter<T> {}

impl<T, R> LineAccessor<T> for R where Self: LineSender<T> + LineGetter<T> {}
