use std::{any::Any, collections::HashMap};

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
    fn unmount(&mut self, globals: &mut Globals);
}

impl<C: Component> AsBoxAny for C {
    #[inline]
    fn as_box_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub struct ComponentRef<T: Component>(u64, std::marker::PhantomData<T>);
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

    fn take(&mut self) -> Box<dyn Component>;
    fn replace(&mut self, component: Box<dyn Component>);
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
    fn take(&mut self) -> Box<dyn Component> {
        self.component.take().unwrap()
    }

    #[inline]
    fn replace(&mut self, component: Box<dyn Component>) {
        self.component = Some(component.as_box_any().downcast::<T>().unwrap());
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

/// UI node storing the `Component` type and surrounding relevant node references.
pub struct ComponentNode<T: Component> {
    parent: UntypedComponentRef,
    children: Vec<UntypedComponentRef>,
    component: Option<Box<T>>,
}

pub struct Globals {
    map: HashMap<u64, Box<dyn InternalNode>>,
}

impl Globals {
    /// Immutably retrieves the `Component` behind a reference.
    #[inline]
    pub fn get<T: Component>(&self, cref: ComponentRef<T>) -> &T {
        self.node(cref).component.as_ref().unwrap()
    }

    /// Mutably retrieves the `Component` behind a reference.
    #[inline]
    pub fn get_mut<T: Component>(&mut self, cref: ComponentRef<T>) -> &mut T {
        self.node_mut(cref).component.as_mut().unwrap()
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
            self.map.remove(&id);
        }
    }

    fn late_unmount_impl(&mut self, cref: impl CRef, v: &mut Vec<u64>) {
        v.push(cref.id());
        let mut component = self.untyped_node_mut(&cref).take();
        component.unmount(self);
        self.untyped_node_mut(&cref).replace(component);

        for child in self
            .map
            .get_mut(&cref.id())
            .expect("invalid reference")
            .children()
            .to_vec()
        {
            self.late_unmount_impl(child, v);
        }
    }

    fn unmount_single(&mut self, cref: &impl CRef) {
        let mut component = self.untyped_node_mut(cref).take();
        component.unmount(self);
        self.untyped_node_mut(cref).replace(component);
        self.map.remove(&cref.id());
    }

    fn unmount_children(&mut self, cref: &impl CRef, reverse: bool) {
        for child in self
            .map
            .get_mut(&cref.id())
            .expect("invalid reference")
            .children()
            .to_vec()
        {
            if reverse {
                self.reverse_unmount(child);
            } else {
                self.unmount(child);
            }
        }
    }

    #[inline]
    fn untyped_node(&self, cref: &impl CRef) -> &Box<dyn InternalNode> {
        self.map.get(&cref.id()).expect("invalid reference")
    }

    #[inline]
    fn untyped_node_mut(&mut self, cref: &impl CRef) -> &mut Box<dyn InternalNode> {
        self.map.get_mut(&cref.id()).expect("invalid reference")
    }
}
