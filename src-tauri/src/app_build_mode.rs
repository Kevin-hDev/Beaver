#[derive(Clone, Copy)]
pub(super) enum AppBuildMode {
    Interactive,
    LiveFixture,
}

impl AppBuildMode {
    pub(super) fn installs_single_instance(self) -> bool {
        matches!(self, Self::Interactive)
    }
}
