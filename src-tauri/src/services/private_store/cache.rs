#[derive(Clone, Copy)]
pub(crate) enum StoreFailure {
    Read,
    Write,
}

pub(crate) enum StoreLoad<T> {
    Missing,
    Ready(T),
    Unavailable(StoreFailure),
}

pub(crate) struct StoreErrorCodes {
    missing: &'static str,
    read: &'static str,
    write: &'static str,
}

impl StoreErrorCodes {
    pub(crate) const fn new(
        missing: &'static str,
        read: &'static str,
        write: &'static str,
    ) -> Self {
        Self {
            missing,
            read,
            write,
        }
    }

    fn failure(&self, failure: StoreFailure) -> String {
        match failure {
            StoreFailure::Read => self.read,
            StoreFailure::Write => self.write,
        }
        .to_string()
    }
}

pub(crate) struct CachedStore<T> {
    value: Option<T>,
    persisted: bool,
    failure: Option<StoreFailure>,
}

impl<T: Clone + Default> CachedStore<T> {
    pub(crate) fn new(load: StoreLoad<T>) -> Self {
        match load {
            StoreLoad::Missing => Self {
                value: Some(T::default()),
                persisted: false,
                failure: None,
            },
            StoreLoad::Ready(value) => Self {
                value: Some(value),
                persisted: true,
                failure: None,
            },
            StoreLoad::Unavailable(failure) => Self {
                value: None,
                persisted: true,
                failure: Some(failure),
            },
        }
    }

    pub(crate) fn value_or_reload(
        &mut self,
        load: impl FnOnce() -> StoreLoad<T>,
        errors: &StoreErrorCodes,
    ) -> Result<&T, String> {
        if self.value.is_none() {
            match load() {
                StoreLoad::Ready(value) => {
                    self.value = Some(value);
                    self.persisted = true;
                    self.failure = None;
                }
                StoreLoad::Missing => {
                    return Err(match self.failure {
                        Some(StoreFailure::Write) => errors.failure(StoreFailure::Write),
                        _ => errors.missing.to_string(),
                    });
                }
                StoreLoad::Unavailable(failure) => {
                    self.failure = Some(failure);
                    return Err(errors.failure(failure));
                }
            }
        }
        self.value.as_ref().ok_or_else(|| errors.read.to_string())
    }

    pub(crate) fn candidate_for_write(
        &mut self,
        load: impl FnOnce() -> StoreLoad<T>,
        errors: &StoreErrorCodes,
    ) -> Result<T, String> {
        match load() {
            StoreLoad::Ready(value) => {
                self.value = Some(value);
                self.persisted = true;
                self.failure = None;
            }
            StoreLoad::Missing if !self.persisted && self.value.is_some() => {}
            StoreLoad::Missing => {
                self.value = None;
                self.persisted = true;
                return Err(match self.failure {
                    Some(StoreFailure::Write) => errors.failure(StoreFailure::Write),
                    _ => errors.missing.to_string(),
                });
            }
            StoreLoad::Unavailable(failure) => {
                self.value = None;
                self.persisted = true;
                self.failure = Some(failure);
                return Err(errors.failure(failure));
            }
        }
        self.value.clone().ok_or_else(|| errors.read.to_string())
    }

    pub(crate) fn commit(&mut self, value: T) {
        self.value = Some(value);
        self.persisted = true;
        self.failure = None;
    }
}
