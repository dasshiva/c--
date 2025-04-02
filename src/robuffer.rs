#[derive(Debug)]
// Read-only buffer
pub struct ROBuffer {
    contents: Vec<u8>,
    offset: usize
}

impl ROBuffer {
    pub fn new(ty: String) -> Result<Self, ()> {
        let mut bytes: Vec<u8> = Vec::new();
        let chars: Vec<char> = ty.chars().collect();
        for ch in chars {
            let b = u32::from(ch);
            if b > 0x7F {
                return Err(())
            }

            bytes.push(b as u8)
        }

        Ok(Self {
            contents: bytes,
            offset: 0
        })
    }

    pub fn next(&mut self) -> Option<u8> {
        if self.contents.len() <= self.offset {
            return None;
        }

        self.offset += 1;
        Some(self.contents[self.offset - 1])
    }

    pub fn rewind(&mut self) {
        if self.offset == 0 {
            panic!("Internal error: Attempt to rewind at offset 0");
        }

        self.offset -= 1;
    }
}

