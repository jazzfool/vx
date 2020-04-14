use {crate::core, reclutch::display as gfx};

pub type LabelRef = core::ComponentRef<Label>;

pub struct Label {
    text: gfx::DisplayText,
}

impl core::ComponentFactory for Label {
    fn new(_globals: &mut core::Globals, _cref: core::ComponentRef<Self>) -> Self {
        Label { text: "".into() }
    }
}

impl core::Component for Label {}
