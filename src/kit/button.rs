use {
    crate::{core, signal},
    reclutch::display as gfx,
};

pub type ButtonRef = core::ComponentRef<Button>;

pub struct Button {
    pub on_click: signal::Signal<()>,
}

impl core::ComponentFactory for Button {
    fn new(_globals: &mut core::Globals, _cref: core::ComponentRef<Self>) -> Self {
        Button {
            on_click: Default::default(),
        }
    }
}

impl core::Component for Button {}
