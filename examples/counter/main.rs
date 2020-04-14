struct Counter {
    count: u32,
    label: vx::kit::LabelRef,
}

impl vx::core::ComponentFactory for Counter {
    fn new(globals: &mut vx::core::Globals, cref: vx::core::ComponentRef<Self>) -> Self {
        let incr: vx::kit::ButtonRef = globals.child(cref);
        let decr: vx::kit::ButtonRef = globals.child(cref);

        globals.listen(
            cref,
            move |globals| &mut globals.get_mut(incr).on_click,
            move |globals, _| {
                globals.get_mut(cref).count += 1;
            },
            Default::default(),
        );

        globals.listen(
            cref,
            move |globals| &mut globals.get_mut(decr).on_click,
            move |globals, _| {
                globals.get_mut(cref).count -= 1;
            },
            Default::default(),
        );

        Counter {
            count: 0,
            label: globals.child(cref),
        }
    }
}

impl vx::core::Component for Counter {
    fn update(&mut self, _globals: &mut vx::core::Globals) {
        println!("update!");
    }
}

// TODO(jazzfool): make a counter
fn main() {
    let (globals, root): (_, vx::core::ComponentRef<Counter>) =
        vx::core::Globals::new(vx::theme::flat::FlatTheme);
}
