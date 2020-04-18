use {
    crate::core,
    std::{collections::HashMap, rc::Rc},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListenerRef(u64);

/// Signal type which broadcasts events to listeners.
pub struct Signal<T: 'static> {
    listeners: HashMap<u64, Rc<dyn Fn(&mut core::Globals, &T)>>,
    next_id: u64,
}

impl<T: 'static> Signal<T> {
    /// Creates a new signal.
    ///
    /// Identical to `Signal::default()`.
    #[inline]
    pub fn new() -> Self {
        Signal {
            listeners: Default::default(),
            next_id: 0,
        }
    }

    /// Adds a listener to the signal.
    #[inline]
    pub fn listen(&mut self, listener: impl Fn(&mut core::Globals, &T) + 'static) -> ListenerRef {
        self.listen_rc(Rc::new(listener))
    }

    /// Removes an existing listener from the signal.
    pub fn remove_listener(&mut self, listener: ListenerRef) {
        self.listeners.remove(&listener.0);
    }

    /// Broadcasts an event to all the listeners.
    pub fn emit(&mut self, globals: &mut core::Globals, event: &T) {
        for listener in self.listeners.values_mut() {
            (*listener)(globals, event);
        }
    }
}

impl<T: 'static> Signal<T> {
    pub(crate) fn listen_rc(
        &mut self,
        listener: Rc<dyn Fn(&mut core::Globals, &T)>,
    ) -> ListenerRef {
        let id = self.next_id;
        self.next_id += 1;
        self.listeners.insert(id, listener);
        ListenerRef(id)
    }
}
