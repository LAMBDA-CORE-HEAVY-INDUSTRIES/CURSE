use core::fmt::Write;

pub struct FmtBuf<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> FmtBuf<'a> {
    pub fn new(buf: &'a mut [u8]) -> Self {
        Self { buf, len: 0 }
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len]).unwrap()
    }
}

impl Write for FmtBuf<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > self.buf.len() {
            return Err(core::fmt::Error);
        }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

pub fn iter_bits_u8(mask: u8) -> impl Iterator<Item = u8> {
    (0..8).filter(move |&t| mask & (1 << t) != 0)
}

pub fn iter_bits_u16(mask: u16) -> impl Iterator<Item = u8> {
    (0..16).filter(move |&t| mask & (1 << t) != 0)
}
