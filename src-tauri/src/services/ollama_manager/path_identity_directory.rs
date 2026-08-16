use super::path_identity_unresolved::UnresolvedDirectory;
#[cfg(any(unix, windows))]
use super::StableDirectoryHandle;
use super::{CanonicalDirectory, NativeDirectoryIdentity, OllamaError, ValidatedPathComponent};
use std::path::{Path, PathBuf};

impl std::fmt::Debug for CanonicalDirectory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CanonicalDirectory")
            .field("path", &self.path)
            .field("identity", &self.identity)
            .field("unresolved", &self.unresolved)
            .finish()
    }
}

impl PartialEq for CanonicalDirectory {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
            && self.identity == other.identity
            && self.unresolved == other.unresolved
    }
}

impl Eq for CanonicalDirectory {}

impl CanonicalDirectory {
    pub(crate) fn synthetic(path: PathBuf, identity: Option<NativeDirectoryIdentity>) -> Self {
        #[cfg(any(unix, windows))]
        return Self::from_native(path, identity, None);
        #[cfg(not(any(unix, windows)))]
        Self::from_native(path, identity)
    }

    pub(crate) fn from_native(
        path: PathBuf,
        identity: Option<NativeDirectoryIdentity>,
        #[cfg(any(unix, windows))] handle: Option<StableDirectoryHandle>,
    ) -> Self {
        Self {
            path,
            identity,
            unresolved: None,
            #[cfg(any(unix, windows))]
            handle,
        }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn identity(&self) -> Option<&NativeDirectoryIdentity> {
        self.identity.as_ref()
    }

    #[cfg(any(unix, windows))]
    pub(crate) fn stable_handle(&self) -> Option<&std::fs::File> {
        self.handle.as_ref().map(|handle| handle.0.as_ref())
    }

    #[cfg(test)]
    pub(crate) fn has_stable_handle(&self) -> bool {
        #[cfg(any(unix, windows))]
        return self.handle.is_some();
        #[cfg(not(any(unix, windows)))]
        false
    }

    pub(crate) fn unresolved_child(&self, component: ValidatedPathComponent) -> Self {
        let path = self.path.join(component.as_os_str());
        let unresolved = Some(match &self.unresolved {
            Some(parent) => parent.child(component),
            None => UnresolvedDirectory::first(self.path.clone(), self.identity.clone(), component),
        });
        #[cfg(any(unix, windows))]
        return Self {
            path,
            identity: None,
            unresolved,
            handle: self.handle.clone(),
        };
        #[cfg(not(any(unix, windows)))]
        Self {
            path,
            identity: None,
            unresolved,
        }
    }

    pub(crate) fn same_unresolved_location(&self, other: &Self) -> Result<bool, OllamaError> {
        match (&self.unresolved, &other.unresolved) {
            (Some(left), Some(right)) => left.same_location(right),
            _ => Ok(false),
        }
    }

    pub(crate) fn unresolved_descendant_of(
        &self,
        parent: &Self,
    ) -> Result<Option<bool>, OllamaError> {
        let Some(child) = self.unresolved.as_ref() else {
            return Ok(None);
        };
        if child.same_anchor(parent.identity()) {
            return Ok(Some(true));
        }
        match parent.unresolved.as_ref() {
            Some(parent) => child.descendant_of(parent).map(Some),
            None => Ok(None),
        }
    }

    pub(crate) fn existing_anchor_path(&self) -> &Path {
        self.unresolved
            .as_ref()
            .map(UnresolvedDirectory::anchor_path)
            .unwrap_or(&self.path)
    }
}
