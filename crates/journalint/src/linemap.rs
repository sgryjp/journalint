use core::cmp::min;

use lsp_types::Position;

#[derive(Debug)]
pub struct LineMap {
    line_offsets: Vec<usize>,
    content_length: usize,
}

impl LineMap {
    pub fn new(content: &str) -> Self {
        let mut line_offsets = vec![0];

        // char_indices() is based on UTF-8 but chumsky consumes the input as a char
        // stream so we need to count chars.
        let mut iter = content.chars().enumerate();
        loop {
            let Some((offset1, ch1)) = iter.next() else {
                break;
            };

            if ch1 == '\r' {
                if let Some((offset2, ch2)) = &iter.next() {
                    if *ch2 == '\n' {
                        line_offsets.push(offset2 + 1);
                    } else {
                        line_offsets.push(offset1 + 1);
                    }
                } else {
                    line_offsets.push(offset1 + 1);
                    break; // finished with CR
                }
            } else if ch1 == '\n' {
                line_offsets.push(offset1 + 1);
            }
        }
        assert!(!line_offsets.is_empty());
        Self {
            line_offsets,
            content_length: content.chars().count(),
        }
    }

    pub fn position_from_offset(&self, offset: usize) -> Position {
        let (line, character) = self._position_from_offset(offset);
        Position { line, character }
    }

    fn _position_from_offset(&self, offset: usize) -> (u32, u32) {
        let line_offsets = &self.line_offsets;

        // Find the largest line-offset that is smaller than the given offset
        assert!(!line_offsets.is_empty());
        for line_index in 1..line_offsets.len() {
            let line_offset = line_offsets[line_index];
            if offset < line_offset {
                let prev_line_offset = line_offsets[line_index - 1];
                return ((line_index - 1) as u32, (offset - prev_line_offset) as u32);
            }
        }

        let final_line_index = line_offsets.len() - 1;
        let final_line_offset = line_offsets.last().unwrap();
        (
            final_line_index as u32,
            (min(offset, self.content_length) - final_line_offset) as u32,
        )
    }

    pub fn offset_from_position(&self, position: &Position) -> usize {
        self._offset_from_position(position.line as usize, position.character as usize)
    }

    fn _offset_from_position(&self, line: usize, character: usize) -> usize {
        assert!(!self.line_offsets.is_empty());
        let x = self
            .line_offsets
            .get(line)
            .unwrap_or(self.line_offsets.last().unwrap());
        min(x + character, self.content_length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let lm = LineMap::new("a\n亜\r\nc");
        assert_eq!(lm.line_offsets, vec![0, 2, 5]);
    }

    #[test]
    fn _position_from_offset() {
        let lm = LineMap::new("a\n亜\r\nc");
        assert_eq!(lm._position_from_offset(0), (0, 0)); // a
        assert_eq!(lm._position_from_offset(1), (0, 1)); // \n
        assert_eq!(lm._position_from_offset(2), (1, 0)); // '亜'
        assert_eq!(lm._position_from_offset(3), (1, 1)); // \r
        assert_eq!(lm._position_from_offset(4), (1, 2)); // \n
        assert_eq!(lm._position_from_offset(5), (2, 0)); // c
        assert_eq!(lm._position_from_offset(6), (2, 1)); // EOS
        assert_eq!(lm._position_from_offset(7), (2, 1)); // EOS + 1
    }

    #[test]
    fn _offset_from_position() {
        let lm = LineMap::new("a\n亜\r\nc"); // '亜' in UTF8: e4 ba 9c
        assert_eq!(lm._offset_from_position(0, 0), 0); // a
        assert_eq!(lm._offset_from_position(0, 1), 1); // \n
        assert_eq!(lm._offset_from_position(1, 0), 2); // '亜'
        assert_eq!(lm._offset_from_position(1, 1), 3); // \r
        assert_eq!(lm._offset_from_position(1, 2), 4); // \n
        assert_eq!(lm._offset_from_position(2, 0), 5); // c
        assert_eq!(lm._offset_from_position(2, 1), 6); // EOS
        assert_eq!(lm._offset_from_position(2, 2), 6); // EOS + 1
    }
}
