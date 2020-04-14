use {crate::core, std::collections::HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListenerRef(u64);

/// Signal type which broadcasts events to listeners.
pub struct Signal<T> {
    listeners: HashMap<u64, Box<dyn FnMut(&mut core::Globals, &T)>>,
    next_id: u64,
}

impl<T> Default for Signal<T> {
    fn default() -> Self {
        Signal {
            listeners: Default::default(),
            next_id: 0,
        }
    }
}

impl<T> Signal<T> {
    /// Creates a new signal.
    ///
    /// Identical to `Signal::default()`.
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a listener to the signal.
    pub fn listen(
        &mut self,
        listener: impl FnMut(&mut core::Globals, &T) + 'static,
    ) -> ListenerRef {
        let id = self.next_id;
        self.next_id += 1;
        self.listeners.insert(id, Box::new(listener));
        ListenerRef(id)
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
