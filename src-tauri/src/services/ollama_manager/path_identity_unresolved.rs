use super::{NativeDirectoryIdentity, OllamaError, ValidatedPathComponent};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct UnresolvedDirectory {
    anchor_path: PathBuf,
    anchor_identity: Option<NativeDirectoryIdentity>,
    components: Vec<ValidatedPathComponent>,
}

impl UnresolvedDirectory {
    pub(super) fn first(
        anchor_path: PathBuf,
        anchor_identity: Option<NativeDirectoryIdentity>,
        component: ValidatedPathComponent,
    ) -> Self {
        Self {
            anchor_path,
            anchor_identity,
            components: vec![component],
        }
    }

    pub(super) fn child(&self, component: ValidatedPathComponent) -> Self {
        let mut next = self.clone();
        next.components.push(component);
        next
    }

    pub(super) fn anchor_path(&self) -> &Path {
        &self.anchor_path
    }

    pub(super) fn same_anchor(&self, identity: Option<&NativeDirectoryIdentity>) -> bool {
        self.anchor_identity.as_ref() == identity && identity.is_some()
    }

    pub(super) fn same_location(&self, other: &Self) -> Result<bool, OllamaError> {
        // Missing suffixes are compared only after one native directory identity anchors them;
        // this preserves filesystem semantics without making path text an authority.
        Ok(self.anchor_identity.is_some()
            && self.anchor_identity == other.anchor_identity
            && same_components(&self.components, &other.components)?)
    }

    pub(super) fn descendant_of(&self, parent: &Self) -> Result<bool, OllamaError> {
        Ok(self.anchor_identity.is_some()
            && self.anchor_identity == parent.anchor_identity
            && self.components.len() > parent.components.len()
            && same_components(
                &self.components[..parent.components.len()],
                &parent.components,
            )?)
    }
}

fn same_components(
    left: &[ValidatedPathComponent],
    right: &[ValidatedPathComponent],
) -> Result<bool, OllamaError> {
    if left.len() != right.len() {
        return Ok(false);
    }
    for (left, right) in left.iter().zip(right) {
        if !same_component(left, right)? {
            return Ok(false);
        }
    }
    Ok(true)
}

#[cfg(unix)]
fn same_component(
    left: &ValidatedPathComponent,
    right: &ValidatedPathComponent,
) -> Result<bool, OllamaError> {
    Ok(left == right)
}

#[cfg(windows)]
fn same_component(
    left: &ValidatedPathComponent,
    right: &ValidatedPathComponent,
) -> Result<bool, OllamaError> {
    super::path_identity_windows::same_component(left.as_os_str(), right.as_os_str())
}
