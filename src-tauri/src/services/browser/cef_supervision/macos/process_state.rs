#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum MacProcessObservation {
    Alive,
    Stopped,
    Unknown,
}
