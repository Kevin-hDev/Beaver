pub(crate) enum StoreLoad<T> {
    Missing,
    Ready(T),
    Unavailable,
}

pub(crate) struct CachedStore<T> {
    value: Option<T>,
    persisted: bool,
}

impl<T: Clone + Default> CachedStore<T> {
    pub(crate) fn new(load: StoreLoad<T>) -> Self {
        match load {
            StoreLoad::Missing => Self {
                value: Some(T::default()),
                persisted: false,
            },
            StoreLoad::Ready(value) => Self {
                value: Some(value),
                persisted: true,
            },
            StoreLoad::Unavailable => Self {
                value: None,
                persisted: true,
            },
        }
    }

    pub(crate) fn value_or_reload(
        &mut self,
        load: impl FnOnce() -> StoreLoad<T>,
        error: &str,
    ) -> Result<&T, String> {
        if self.value.is_none() {
            let StoreLoad::Ready(value) = load() else {
                return Err(error.to_string());
            };
            self.value = Some(value);
            self.persisted = true;
        }
        self.value.as_ref().ok_or_else(|| error.to_string())
    }

    pub(crate) fn candidate_for_write(
        &mut self,
        load: impl FnOnce() -> StoreLoad<T>,
        error: &str,
    ) -> Result<T, String> {
        match load() {
            StoreLoad::Ready(value) => {
                self.value = Some(value);
                self.persisted = true;
            }
            StoreLoad::Missing if !self.persisted && self.value.is_some() => {}
            StoreLoad::Missing | StoreLoad::Unavailable => {
                self.value = None;
                self.persisted = true;
                return Err(error.to_string());
            }
        }
        self.value.clone().ok_or_else(|| error.to_string())
    }

    pub(crate) fn commit(&mut self, value: T) {
        self.value = Some(value);
        self.persisted = true;
    }
}
