struct Counter {
    count: u32,
    btn: vx::kit::ButtonRef,
}

impl vx::core::ComponentFactory for Counter {
    fn new(globals: &mut vx::core::Globals, cref: vx::core::ComponentRef<Self>) -> Self {
        let btn: vx::kit::ButtonRef = globals.child(cref);

        globals.listen(globals.get(btn).on_click, cref, move |globals, _| {
            globals.get_mut(cref).count += 1;
            globals.update(cref, vx::core::Repaint::No, vx::core::Propagate::No);
        });

        Counter { count: 0, btn }
    }
}

impl vx::core::Component for Counter {
    fn unmount(&mut self, _globals: &mut vx::core::Globals) {
        println!("unmount");
    }

    fn update(&mut self, _globals: &mut vx::core::Globals) {
        println!("count = {}", self.count);
    }
}

// TODO(jazzfool): make a counter
fn main() {
    let (mut globals, root): (_, vx::core::ComponentRef<Counter>) =
        vx::core::Globals::new(vx::theme::flat::FlatTheme);
    globals.update(root, Default::default(), Default::default());

    for _ in 0..1000 {
        globals.emit(globals.get(globals.get(root).btn).on_click, &());
    }
}
