use {
    crate::{signal, theme},
    reclutch::display as gfx,
    std::{any::Any, collections::HashMap, rc::Rc},
};

/// Core component trait, implemented by all distinct elements of a UI.
pub trait Component: AsBoxAny + 'static {
    /// Invoked right before the component is removed/deleted.
    ///
    /// The are four possible environments when this is called;
    /// - Parent node doesn't exist.
    /// - Child nodes don't exist.
    /// - Neither parent nor child nodes exist.
    /// - Both parent and children nodes exist.
    ///
    /// The first is caused if this `unmount` is a result of a parent being unmounted (indirect unmount).
    ///
    /// The second is caused if this `unmount` is a result of a `reverse_unmount` (direct unmount).
    ///
    /// The third is caused if this `unmount` is a result of a parent being late unmounted (indrect unmount).
    ///
    /// The fourth is caused if this `unmount` is a result of a regular `unmount` (direct unmount) *or* `late_unmount` (direct or indirect unmount).
    #[inline]
    fn unmount(&mut self, _globals: &mut Globals) {}

    /// Invoked during rendering.
    ///
    /// This should return a list of display commands that should be used to display this component.
    #[inline]
    fn display(&mut self) -> Vec<gfx::DisplayCommand> {
        Default::default()
    }

    /// Invoked by [`Globals::update`](Globals::update), either as a result of propagation or directly.
    ///
    /// Update logic should be placed here.
    ///
    /// Do not emit any events here.
    #[inline]
    fn update(&mut self, _globals: &mut Globals) {}
}

impl<C: Component> AsBoxAny for C {
    #[inline]
    fn as_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

/// Implemented by components capable of constructing themselves.
pub trait ComponentFactory: Sized + Component {
    /// Constructs a new component of type `Self`.
    ///
    /// `cref` is the reference to self component within `globals`.
    fn new(globals: &mut Globals, cref: ComponentRef<Self>) -> Self;
}

/// Strongly-typed reference to a component.
#[derive(Derivative)]
#[derivative(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
// don't constraint T
#[derivative(Debug(bound = ""))]
#[derivative(Clone(bound = ""))]
#[derivative(Copy(bound = ""))]
#[derivative(PartialEq(bound = ""))]
#[derivative(Eq(bound = ""))]
#[derivative(PartialOrd(bound = ""))]
#[derivative(Ord(bound = ""))]
#[derivative(Hash(bound = ""))]
pub struct ComponentRef<T: Component>(u64, std::marker::PhantomData<T>);

/// Untyped reference to a component.
///
/// Prefer the strongly-typed variant, [`ComponentRef`](ComponentRef), where possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UntypedComponentRef(u64);

/// Implemented by any type which references a node, strongly-typed or not.
pub trait CRef {
    /// Returns the underlying ID of the node.
    fn id(&self) -> u64;
}

impl<T: Component> CRef for ComponentRef<T> {
    #[inline]
    fn id(&self) -> u64 {
        self.0
    }
}

impl CRef for UntypedComponentRef {
    #[inline]
    fn id(&self) -> u64 {
        self.0
    }
}

impl UntypedComponentRef {
    /// Attaches a type to the component reference.
    ///
    /// # Warning
    /// Call this sparingly and cautiously. It will cause a `panic` if an incorrect type is provided.
    #[inline]
    pub fn to_typed<T: Component>(self) -> ComponentRef<T> {
        ComponentRef(self.0, Default::default())
    }
}

#[doc(hidden)]
pub trait AsBoxAny {
    fn as_box_any(self: Box<Self>) -> Box<dyn Any>;
}

/// Public interface for a UI node.
pub trait Node {
    /// Returns a reference to the parent component.
    fn parent(&self) -> UntypedComponentRef;
    /// Returns a list of references to the child components.
    fn children(&self) -> &[UntypedComponentRef];
}

trait InternalNode: Node {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_node(&self) -> &dyn Node;
    fn as_node_mut(&mut self) -> &mut dyn Node;

    fn take(&mut self) -> Box<dyn Component>;
    fn replace(&mut self, component: Box<dyn Component>);
    fn is_taken(&self) -> bool;

    fn detach_listeners(&mut self, globals: &mut Globals);
    fn repaint(&mut self);
    fn push_child(&mut self, child: UntypedComponentRef);
}

impl<T: Component> InternalNode for ComponentNode<T> {
    #[inline]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[inline]
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    #[inline]
    fn as_node(&self) -> &dyn Node {
        self
    }

    #[inline]
    fn as_node_mut(&mut self) -> &mut dyn Node {
        self
    }

    #[inline]
    fn take(&mut self) -> Box<dyn Component> {
        Box::new(self.component.take().unwrap())
    }

    #[inline]
    fn replace(&mut self, component: Box<dyn Component>) {
        self.component = Some(*component.as_box_any().downcast::<T>().unwrap());
    }

    #[inline]
    fn is_taken(&self) -> bool {
        self.component.is_none()
    }

    #[inline]
    fn detach_listeners(&mut self, globals: &mut Globals) {
        for listener in &mut self.listeners {
            listener.detach(globals);
        }
    }

    #[inline]
    fn repaint(&mut self) {
        self.cmds.repaint();
    }

    #[inline]
    fn push_child(&mut self, child: UntypedComponentRef) {
        self.children.push(child);
    }
}

impl<T: Component> Node for ComponentNode<T> {
    #[inline]
    fn parent(&self) -> UntypedComponentRef {
        self.parent
    }

    #[inline]
    fn children(&self) -> &[UntypedComponentRef] {
        &self.children
    }
}

struct ListenerPair {
    listener: signal::ListenerRef,
    signal: u64,
}

impl ListenerPair {
    fn detach(&self, globals: &mut Globals) {
        if let Some(signal) = globals.signal_map.get_mut(&self.signal).unwrap().as_mut() {
            signal.detach(self.listener);
        } else {
            globals.listener_removal.push(self.listener);
        }
    }
}

/// UI node storing the `Component` type and surrounding relevant node references.
pub struct ComponentNode<T: Component> {
    parent: UntypedComponentRef,
    children: Vec<UntypedComponentRef>,
    component: Option<T>,
    listeners: Vec<ListenerPair>,
    cmds: gfx::CommandGroup,
}

/// Whether a repaint should be scheduled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Repaint {
    Yes,
    No,
}

impl Default for Repaint {
    fn default() -> Self {
        Repaint::Yes
    }
}

/// Whether an invocation should be recursively propagated to children.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Propagate {
    Yes,
    No,
}

impl Default for Propagate {
    fn default() -> Self {
        Propagate::Yes
    }
}

/// Whether an update should be invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Update {
    Yes(Repaint, Propagate),
    No,
}

impl Default for Update {
    fn default() -> Self {
        Update::Yes(Default::default(), Default::default())
    }
}

#[derive(Derivative)]
#[derivative(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derivative(Debug(bound = ""))]
#[derivative(Clone(bound = ""))]
#[derivative(Copy(bound = ""))]
#[derivative(PartialEq(bound = ""))]
#[derivative(Eq(bound = ""))]
#[derivative(PartialOrd(bound = ""))]
#[derivative(Ord(bound = ""))]
#[derivative(Hash(bound = ""))]
pub struct SignalRef<T>(u64, std::marker::PhantomData<T>);

impl<T> SignalRef<T> {
    #[inline]
    pub(crate) fn null() -> Self {
        SignalRef(std::u64::MAX, Default::default())
    }
}

trait InternalSignal {
    fn emit(&mut self, globals: &mut Globals, event: &dyn Any);
    fn listen(&mut self, listener: &dyn Any) -> signal::ListenerRef;
    fn detach(&mut self, listener: signal::ListenerRef);
}

impl<T: 'static> InternalSignal for signal::Signal<T> {
    #[inline]
    fn emit(&mut self, globals: &mut Globals, event: &dyn Any) {
        self.emit(globals, event.downcast_ref::<T>().unwrap())
    }

    fn listen(&mut self, listener: &dyn Any) -> signal::ListenerRef {
        self.listen_rc(Rc::clone(
            listener
                .downcast_ref::<Rc<dyn Fn(&mut Globals, &T)>>()
                .unwrap(),
        ))
    }

    #[inline]
    fn detach(&mut self, listener: signal::ListenerRef) {
        self.remove_listener(listener);
    }
}

pub struct Globals {
    pub on_theme_changed: SignalRef<()>,
    map: HashMap<u64, Box<dyn InternalNode>>,
    signal_map: HashMap<u64, Option<Box<dyn InternalSignal>>>,
    listener_removal: Vec<signal::ListenerRef>,
    next_component_id: u64,
    next_signal_id: u64,
    theme: Box<dyn theme::Theme>,
}

impl Globals {
    /// Creates a new `Globals` with a root component and initial theme.
    pub fn new<T: ComponentFactory>(theme: impl theme::Theme + 'static) -> (Self, ComponentRef<T>) {
        let mut globals = Globals {
            on_theme_changed: SignalRef::null(),

            map: Default::default(),
            signal_map: Default::default(),
            listener_removal: Default::default(),
            next_component_id: 0,
            next_signal_id: 0,
            theme: Box::new(theme),
        };

        globals.on_theme_changed = globals.signal();

        let root = ComponentRef(globals.next_component_id, Default::default());
        globals.next_component_id += 1;
        globals.map.insert(
            root.0,
            Box::new(ComponentNode::<T> {
                parent: UntypedComponentRef(root.0),
                children: Vec::new(),
                component: None,
                listeners: Vec::new(),
                cmds: Default::default(),
            }),
        );

        globals.node_mut(root).component = Some(T::new(&mut globals, root));

        (globals, root)
    }

    /// Immutably retrieves the `Component` behind a reference.
    #[inline]
    pub fn get<T: Component>(&self, cref: ComponentRef<T>) -> &T {
        self.node(cref)
            .component
            .as_ref()
            .expect("a reference to the component is already being used")
    }

    /// Mutably retrieves the `Component` behind a reference.
    #[inline]
    pub fn get_mut<T: Component>(&mut self, cref: ComponentRef<T>) -> &mut T {
        self.node_mut(cref)
            .component
            .as_mut()
            .expect("a reference to the component is already being used")
    }

    /// Attempts to immutably retrieve the `Component` behind a reference, returning `None` if it failed.
    ///
    /// It will return `None` if;
    /// - Component type doesn't match.
    /// - The component is being used somewhere in the stack trace.
    /// - The component reference is invalid (component has been unmounted).
    #[inline]
    pub fn try_get<T: Component>(&self, cref: ComponentRef<T>) -> Option<&T> {
        self.try_node(cref)?.component.as_ref()
    }

    /// Attempts to mutably retrieve the `Component` behind a reference, returning `None` if it failed.
    #[inline]
    pub fn try_get_mut<T: Component>(&mut self, cref: ComponentRef<T>) -> Option<&mut T> {
        self.try_node_mut(cref)?.component.as_mut()
    }

    /// Immutably retrieves the `ComponentNode` behind a reference.
    pub fn node<T: Component>(&self, cref: ComponentRef<T>) -> &ComponentNode<T> {
        self.map
            .get(&cref.0)
            .expect("invalid reference")
            .as_any()
            .downcast_ref::<ComponentNode<T>>()
            .expect("mismatching reference type")
    }

    /// Mutably retrieves the `ComponentNode` behind a reference.
    pub fn node_mut<T: Component>(&mut self, cref: ComponentRef<T>) -> &mut ComponentNode<T> {
        self.map
            .get_mut(&cref.0)
            .expect("invalid reference")
            .as_any_mut()
            .downcast_mut::<ComponentNode<T>>()
            .expect("mismatching reference type")
    }

    /// Attempts to immutably retrieve the `Component` behind a reference, returning `None` if it failed.
    pub fn try_node<T: Component>(&self, cref: ComponentRef<T>) -> Option<&ComponentNode<T>> {
        self.map
            .get(&cref.0)?
            .as_any()
            .downcast_ref::<ComponentNode<T>>()
    }

    /// Attempts to mutably retrieve the `ComponentNode` behind a reference, returning `None` if it failed.
    pub fn try_node_mut<T: Component>(
        &mut self,
        cref: ComponentRef<T>,
    ) -> Option<&mut ComponentNode<T>> {
        self.map
            .get_mut(&cref.0)?
            .as_any_mut()
            .downcast_mut::<ComponentNode<T>>()
    }

    /// Returns an immutable dynamic reference to a node behind a component reference.
    #[inline]
    pub fn untyped_node(&self, cref: impl CRef) -> &dyn Node {
        self.untyped_internal_node(&cref).as_node()
    }

    /// Returns a mutable dynamic reference to a node behind a component reference.
    #[inline]
    pub fn untyped_node_mut(&mut self, cref: impl CRef) -> &mut dyn Node {
        self.untyped_internal_node_mut(&cref).as_node_mut()
    }

    /// Returns `true` if the provided reference is valid (hasn't been unmounted), otherwise `false`.
    #[inline]
    pub fn is_valid(&self, cref: impl CRef) -> bool {
        self.map.contains_key(&cref.id())
    }

    /// Returns `true` if the `Component` isn't in the stack trace and is available, otherwise `false`.
    ///
    /// This check is a superset of `is_valid`, in that if this returns `true` then `is_valid` must also return `true` too.
    #[inline]
    pub fn is_available(&self, cref: impl CRef) -> bool {
        self.map
            .get(&cref.id())
            .and_then(|x| Some(!x.is_taken()))
            .unwrap_or(false)
    }

    /// Returns `true` if the `Component` is of type `T`, otherwise `false`.
    ///
    /// This check is a superset of `is_valid`, in that if this returns `true` then `is_valid` must also return `true` too.
    #[inline]
    pub fn is_of_type<T: Component>(&self, cref: ComponentRef<T>) -> bool {
        self.map
            .get(&cref.id())
            .and_then(|x| Some(x.type_id() == std::any::TypeId::of::<T>()))
            .unwrap_or(false)
    }

    /// Unmounts and removes a component node (and it's children).
    ///
    /// If you require access to parent or children from within [component unmount](Component::unmount), consider using [`late_unmount`](Globals::late_unmount) instead.
    #[inline]
    pub fn unmount(&mut self, cref: impl CRef) {
        self.unmount_single(&cref);
        self.unmount_children(&cref, false);
    }

    /// Same as [`unmount`](Globals::unmount), however children are unmounted *before* the component.
    #[inline]
    pub fn reverse_unmount(&mut self, cref: impl CRef) {
        self.unmount_children(&cref, true);
        self.unmount_single(&cref);
    }

    /// Same as [`unmount`](Globals::unmount), however everything is erased after all the `unmount` callbacks have been made.
    ///
    /// This gives the `unmount` callbacks most flexibility in terms of the existence of parent/children but is the slowest unmount method (two iterations over local UI tree instead of one).
    pub fn late_unmount(&mut self, cref: impl CRef) {
        let mut v = Vec::new();
        self.late_unmount_impl(cref, &mut v);
        for id in v {
            if let Some(mut node) = self.map.remove(&id) {
                node.detach_listeners(self);
            }
        }
    }

    /// Creates a new component as a child of an existing component.
    pub fn child<T: ComponentFactory>(&mut self, pcref: impl CRef) -> ComponentRef<T> {
        let cref = ComponentRef(self.next_component_id, Default::default());
        self.next_component_id += 1;

        self.untyped_internal_node_mut(&pcref)
            .push_child(UntypedComponentRef(cref.0));
        self.map.insert(
            cref.0,
            Box::new(ComponentNode::<T> {
                parent: UntypedComponentRef(pcref.id()),
                children: Vec::new(),
                component: None,
                listeners: Vec::new(),
                cmds: Default::default(),
            }),
        );

        self.node_mut(cref).component = Some(T::new(self, cref));

        cref
    }

    /// Invokes an update for a specified component, optionally recursively propagating to children and scheduling a repaint.
    pub fn update(&mut self, cref: impl CRef, repaint: Repaint, propagate: Propagate) {
        let mut component = self.untyped_internal_node_mut(&cref).take();
        component.update(self);
        self.untyped_internal_node_mut(&cref).replace(component);

        let node = self.untyped_internal_node_mut(&cref);

        if Repaint::Yes == repaint {
            node.repaint();
        }

        if Propagate::Yes == propagate {
            for child in node.children().to_vec() {
                self.update(child, repaint, propagate);
            }
        }
    }

    /// Returns a new painter from the current theme.
    #[inline]
    pub fn painter<T: Component>(&self, p: &'static str) -> theme::Painter<T> {
        theme::get_painter(self.theme.as_ref(), p)
    }

    /// Changes the current theme.
    ///
    /// Components will only update their painters if they correctly handle `on_theme_changed`.
    pub fn set_theme(&mut self, theme: impl theme::Theme + 'static) {
        self.theme = Box::new(theme);
        self.emit(self.on_theme_changed, &());
    }

    /// Creates a new signal.
    pub fn signal<T: 'static>(&mut self) -> SignalRef<T> {
        let sref = SignalRef(self.next_signal_id, Default::default());
        self.next_signal_id += 1;
        self.signal_map
            .insert(sref.0, Some(Box::new(signal::Signal::<T>::new())));
        sref
    }

    /// Emits an event for a signal.
    pub fn emit<T: 'static>(&mut self, sref: SignalRef<T>, event: &T) {
        if let Some(mut signal) = self.signal_map.get_mut(&sref.0).and_then(|x| x.take()) {
            signal.emit(self, event);
            for listener in std::mem::take(&mut self.listener_removal) {
                signal.detach(listener);
            }
            *self.signal_map.get_mut(&sref.0).unwrap() = Some(signal);
        }
    }

    /// Adds a managed listener to a signal.
    ///
    /// "Managed" implies that the listener will be removed when `cref` is unmounted.
    pub fn listen<T: 'static, C: Component>(
        &mut self,
        sref: SignalRef<T>,
        cref: ComponentRef<C>,
        listener: impl Fn(&mut Globals, &T) + 'static,
    ) {
        let listener: Rc<dyn Fn(&mut Globals, &T)> = Rc::new(listener);
        let listener = self
            .signal_map
            .get_mut(&sref.0)
            .expect("invalid signal ref")
            .as_mut()
            .expect("signal already borrowed (call trace is mostly likely from a listener for this signal)")
            .listen(&listener);
        self.node_mut(cref).listeners.push(ListenerPair {
            listener,
            signal: sref.0,
        })
    }
}

impl Globals {
    fn late_unmount_impl(&mut self, cref: impl CRef, v: &mut Vec<u64>) {
        v.push(cref.id());
        let mut component = self.untyped_internal_node_mut(&cref).take();
        component.unmount(self);
        self.untyped_internal_node_mut(&cref).replace(component);

        for child in self.untyped_internal_node(&cref).children().to_vec() {
            self.late_unmount_impl(child, v);
        }
    }

    fn unmount_single(&mut self, cref: &impl CRef) {
        let mut component = self.untyped_internal_node_mut(cref).take();
        component.unmount(self);
        self.untyped_internal_node_mut(cref).replace(component);
        if let Some(mut node) = self.map.remove(&cref.id()) {
            node.detach_listeners(self);
        }
    }

    fn unmount_children(&mut self, cref: &impl CRef, reverse: bool) {
        if !self.map.contains_key(&cref.id()) {
            return;
        }

        for child in self.untyped_internal_node(cref).children().to_vec() {
            if self.map.contains_key(&child.0) {
                if reverse {
                    self.reverse_unmount(child);
                } else {
                    self.unmount(child);
                }
            }
        }
    }

    #[inline]
    fn untyped_internal_node(&self, cref: &impl CRef) -> &Box<dyn InternalNode> {
        self.map.get(&cref.id()).expect("invalid reference")
    }

    #[inline]
    fn untyped_internal_node_mut(&mut self, cref: &impl CRef) -> &mut Box<dyn InternalNode> {
        self.map.get_mut(&cref.id()).expect("invalid reference")
    }
}

impl Drop for Globals {
    fn drop(&mut self) {
        let keys: Vec<_> = self.map.keys().map(|x| x.clone()).collect();
        for key in keys {
            if self.map.contains_key(&key) {
                self.unmount(UntypedComponentRef(key));
            }
        }
    }
}
