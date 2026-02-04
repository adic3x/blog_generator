#[derive(Debug, PartialEq)]
pub struct Offsets {
    pub left:   usize,
    pub right:  usize,
    pub length: usize,
}

pub trait TrimOffsets {
    fn trim_offsets(&self) -> Offsets;
}

impl TrimOffsets for str {
    fn trim_offsets(&self) -> Offsets {
        let trimmed = self.trim();
        let left = trimmed.as_ptr() as usize - self.as_ptr() as usize;
        let right = self.len() - trimmed.len() - left;
        Offsets { left, right, length: trimmed.len() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_offsets() {
        let cases = [
            (Offsets { left: 3, right: 3, length: 5 }, "   hello   "),
            (Offsets { left: 0, right: 0, length: 5 }, "hello"),
            (Offsets { left: 4, right: 0, length: 5 }, "    world"),
            (Offsets { left: 0, right: 4, length: 4 }, "rust    "),
            (Offsets { left: 3, right: 3, length: 5 }, "\n\t space \r\n"),
            (Offsets { left: 0, right: 0, length: 0 }, ""),
            (Offsets { left: 0, right: 3, length: 0 }, "   "),
            (Offsets { left: 1, right: 1, length: 2 }, " ї "),
            (Offsets { left: 0, right: 0, length: 9 }, "🦀 🦀"),
            (Offsets { left: 2, right: 2, length: 4 }, "  🦀  "),
        ];

        cases.iter().enumerate().for_each(|(i, (expected, input))| {
            let result = input.trim_offsets();
            
            if result != *expected {
                panic!("\n[Trim #{i} failed]\nInput: {input:?}\nActual:   {result:?}\nExpected: {expected:?}\n");
            }
        });
    }
}