# vx

## Yet another GUI experiment of mine

## Goals

- No `Rc<RefCell<..>>` whatsoever.
- No inherently `unsafe` code whatsoever.
- No non-trivial or non-derive macros whatsoever.

In essence, `vx` aims to be written in only the most _pure_ form of Rust. No interior mutability, 100% safety, and no fancy macros.

```rust
struct Counter {
    count: u32,
}

impl ComponentFactory for Counter {
    fn new(globals: &mut Globals, cref: ComponentRef<Self>) -> Self {
        let btn: ButtonRef = globals.child(cref);

        globals.listen(globals.get(btn).on_click, cref, move |globals, _| {
            globals.get_mut(cref).count += 1;
            globals.update(cref, Repaint::No, Propagate::No);
        });

        Counter { count: 0 }
    }
}

impl Component for Counter {
    fn update(&mut self, _globals: &mut Globals) {
        println!("Count: {}", self.count);
    }
}
```

Due to how it's designed, you can even have graph-like widget relatives;

```rust
let parent: UntypedComponentRef = globals.parent(cref);
let grandparent: UntypedComponentRef = globals.untyped_node(parent).parent();

let parent_idx = globals.untyped_node(grandparent)
    .children()
    .iter()
    .position(|x| *x == parent)
    .expect("this will never fail");
```

You can even get a typed reference if you're absolutely certain of the parent type;

```rust
let parent: UntypedComponentRef = globals.parent(cref);
let parent: ComponentRef<FooComponent> = parent.to_typed();
let parent: &mut FooComponent = globals.get_mut(parent);

// We now have a mutable reference to our parent
```

Much like other UI libraries, there are downsides to this kind of implementation;

- The global object moves things in and out for processing. This means you can't just do whatever you want whenever you want. The object you may be referencing may be "in use".
- Component references aren't aware of deletion state. A reference may very well be invalid and thus cause a `panic` when used.

## License

VX is licensed under either

- [Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
- [MIT](http://opensource.org/licenses/MIT)

at your choosing.
