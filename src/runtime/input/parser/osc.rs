pub(super) fn starts_with_osc(buffer: &[u8]) -> bool {
    buffer.starts_with(b"\x1b]")
}

pub(super) fn end(buffer: &[u8]) -> Option<usize> {
    let mut index = 0;
    while index < buffer.len() {
        if buffer[index] == b'\x07' {
            return Some(index + 1);
        }
        if buffer[index] == b'\x1b' && buffer.get(index + 1) == Some(&b'\\') {
            return Some(index + 2);
        }
        index += 1;
    }
    None
}
