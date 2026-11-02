//! What happens in the users list, in the past tense.

/// Events for `UsersBloc`.
#[derive(Clone, Debug)]
pub enum UsersEvent {
    /// A page of the list was asked for: the first on opening it, another from its pagination.
    PageRequested {
        /// The page, from 1.
        page: u32,
    },
}
