use crossterm::event::Event;

use crate::runtime::kitty_dnd::KittyDndEvent;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::runtime) enum RuntimeInputEvent {
    Terminal(Event),
    KittyDnd(KittyDndEvent),
}
