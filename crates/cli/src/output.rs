use std::io::{self, Write};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub enum Output {
    Inherit,
    Pipe(Arc<Mutex<FnMut(&str) + Send + 'static>>),
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stderr().write_all(buf)?;
        io::stderr().write_all(b"\n")?;

        if let Output::Pipe(f) = self {
            let mut f = f.lock().unwrap();
            let s = String::from_utf8_lossy(buf);
            (&mut *f)(&s);
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stderr().flush()
    }
}
