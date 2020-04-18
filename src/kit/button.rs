use {
    crate::{core, signal, theme},
    reclutch::display as gfx,
};

pub type ButtonRef = core::ComponentRef<Button>;

pub struct Button {
    pub on_click: core::SignalRef<()>,
    painter: theme::Painter<Self>,
}

impl core::ComponentFactory for Button {
    fn new(globals: &mut core::Globals, _cref: core::ComponentRef<Self>) -> Self {
        Button {
            on_click: globals.signal(),
            painter: globals.painter(theme::painters::BUTTON),
        }
    }
}

impl core::Component for Button {
    #[inline]
    fn display(&mut self) -> Vec<gfx::DisplayCommand> {
        theme::paint(self, |o| &mut o.painter)
    }
}
