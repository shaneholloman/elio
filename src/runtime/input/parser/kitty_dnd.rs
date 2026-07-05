use crate::runtime::kitty_dnd::{self, KittyDndEvent};

use super::osc;

#[derive(Default)]
pub(super) struct Parser {
    state: kitty_dnd::Osc72State,
}

impl Parser {
    pub(super) fn parse(&mut self, buffer: &mut Vec<u8>) -> Option<KittyDndEvent> {
        if !buffer.starts_with(b"\x1b]72;") {
            return None;
        }
        let end = osc::end(buffer)?;
        let sequence: Vec<u8> = buffer.drain(..end).collect();
        kitty_dnd::parse_osc72_with_state(&sequence, &mut self.state)
    }
}
