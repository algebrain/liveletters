#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    FriendsOnly,
    MembersOnly,
    PrivateCommunity,
}
