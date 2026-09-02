#[derive(Clone, Copy)]
pub(crate) struct DispatchTrace<'a> {
    pub session_id: &'a str,
    pub request_id: Option<&'a str>,
}
