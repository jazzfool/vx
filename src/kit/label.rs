use {
    crate::{core, theme},
    reclutch::display as gfx,
};

pub type LabelRef = core::ComponentRef<Label>;

pub struct Label {
    text: gfx::DisplayText,
    painter: theme::Painter<Self>,
    cref: LabelRef,
}

impl core::ComponentFactory for Label {
    fn new(globals: &mut core::Globals, cref: core::ComponentRef<Self>) -> Self {
        Label {
            text: "".into(),
            painter: globals.painter(theme::painters::LABEL),
            cref,
        }
    }
}

impl core::Component for Label {
    #[inline]
    fn display(&mut self) -> Vec<gfx::DisplayCommand> {
        theme::paint(self, |o| &mut o.painter)
    }
}

impl Label {
    pub fn set_text(&mut self, globals: &mut core::Globals, text: impl Into<gfx::DisplayText>) {
        self.text = text.into();
        globals.update(self.cref, core::Repaint::Yes, core::Propagate::No);
    }

    #[inline]
    pub fn text(&self) -> gfx::DisplayText {
        self.text.clone()
    }
}
