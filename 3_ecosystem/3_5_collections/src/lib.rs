use std::collections::BTreeMap;
use std::num::NonZeroU64;
use std::sync::Arc;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UserId(NonZeroU64);

impl UserId {
    pub fn new(value: u64) -> Result<Self, UserIdError> {
        let Some(value) = NonZeroU64::new(value) else {
            return Err(UserIdError::ZeroValue);
        };

        Ok(Self(value))
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum UserIdError {
    #[error("failed to construct UserId in UserId::new: value must be non-zero")]
    ZeroValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nickname(String);

impl Nickname {
    pub fn new(value: impl Into<String>) -> Result<Self, NicknameError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(NicknameError::EmptyValue);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum NicknameError {
    #[error("failed to construct Nickname in Nickname::new: value must not be blank")]
    EmptyValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NicknameQuery(String);

impl NicknameQuery {
    pub fn new(value: impl Into<String>) -> Result<Self, NicknameQueryError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(NicknameQueryError::EmptyValue);
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum NicknameQueryError {
    #[error(
        "failed to construct NicknameQuery in NicknameQuery::new: value \
         must not be blank"
    )]
    EmptyValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    id: UserId,
    nickname: Nickname,
}

impl User {
    pub fn new(id: UserId, nickname: Nickname) -> Self {
        Self { id, nickname }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn nickname(&self) -> &Nickname {
        &self.nickname
    }
}

pub trait UsersRepository {
    fn get_user_by_id(&self, user_id: UserId) -> Option<User>;

    fn get_users_by_ids(&self, user_ids: &[UserId]) -> Vec<User>;

    fn search_user_ids_by_nickname(&self, nickname_query: &NicknameQuery) -> Vec<UserId>;
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum UsersRepositoryError {
    #[error(
        "failed to construct ImmutableUsersRepository in \
         ImmutableUsersRepository::new: duplicate user id {0:?}"
    )]
    DuplicateUserId(UserId),
}

#[derive(Debug, Clone, Default)]
pub struct ImmutableUsersRepository {
    users: PersistentUsersMap,
}

impl ImmutableUsersRepository {
    pub fn new(users: impl IntoIterator<Item = User>) -> Result<Self, UsersRepositoryError> {
        let mut users_map = PersistentUsersMap::new();

        for user in users {
            let user_id = user.id();
            if users_map.contains_key(&user_id) {
                return Err(UsersRepositoryError::DuplicateUserId(user_id));
            }

            users_map = users_map.insert(user_id, user);
        }

        Ok(Self { users: users_map })
    }
}

impl UsersRepository for ImmutableUsersRepository {
    fn get_user_by_id(&self, user_id: UserId) -> Option<User> {
        self.users.get(&user_id).cloned()
    }

    fn get_users_by_ids(&self, user_ids: &[UserId]) -> Vec<User> {
        user_ids
            .iter()
            .filter_map(|user_id| self.users.get(user_id).cloned())
            .collect()
    }

    fn search_user_ids_by_nickname(&self, nickname_query: &NicknameQuery) -> Vec<UserId> {
        let user_ids: Vec<UserId> = self
            .users
            .values()
            .filter(|user| user.nickname().as_str().contains(nickname_query.as_str()))
            .map(User::id)
            .collect();
        user_ids
    }
}

#[derive(Debug, Clone, Default)]
struct PersistentUsersMap {
    inner: Arc<BTreeMap<UserId, User>>,
}

impl PersistentUsersMap {
    fn new() -> Self {
        Self::default()
    }

    fn contains_key(&self, user_id: &UserId) -> bool {
        self.inner.contains_key(user_id)
    }

    fn get(&self, user_id: &UserId) -> Option<&User> {
        self.inner.get(user_id)
    }

    fn values(&self) -> impl Iterator<Item = &User> {
        self.inner.values()
    }

    fn insert(&self, user_id: UserId, user: User) -> Self {
        let mut users = (*self.inner).clone();
        users.insert(user_id, user);

        Self {
            inner: Arc::new(users),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(user_id: u64, nickname: &str) -> User {
        User::new(
            UserId::new(user_id).unwrap(),
            Nickname::new(nickname).unwrap(),
        )
    }

    fn repository() -> ImmutableUsersRepository {
        ImmutableUsersRepository::new(vec![
            user(1, "alice"),
            user(2, "bob"),
            user(3, "alice_wonder"),
        ])
        .unwrap()
    }

    #[test]
    fn get_user_by_id_returns_matching_user() {
        let repository = repository();
        let found = repository.get_user_by_id(UserId::new(2).unwrap());

        assert_eq!(found, Some(user(2, "bob")));
    }

    #[test]
    fn get_users_by_ids_returns_existing_users_in_requested_order() {
        let repository = repository();
        let ids = vec![
            UserId::new(2).unwrap(),
            UserId::new(99).unwrap(),
            UserId::new(1).unwrap(),
            UserId::new(2).unwrap(),
        ];

        let users = repository.get_users_by_ids(&ids);

        assert_eq!(
            users,
            vec![user(2, "bob"), user(1, "alice"), user(2, "bob")]
        );
    }

    #[test]
    fn search_user_ids_by_nickname_returns_matching_ids() {
        let repository = repository();
        let query = NicknameQuery::new("alice").unwrap();

        let user_ids = repository.search_user_ids_by_nickname(&query);

        assert_eq!(
            user_ids,
            vec![UserId::new(1).unwrap(), UserId::new(3).unwrap()]
        );
    }

    #[test]
    fn repository_creation_fails_on_duplicate_user_id() {
        let error =
            ImmutableUsersRepository::new(vec![user(7, "first"), user(7, "second")]).unwrap_err();

        assert_eq!(
            error,
            UsersRepositoryError::DuplicateUserId(UserId::new(7).unwrap())
        );
    }

    #[test]
    fn nickname_query_rejects_empty_value() {
        let error = NicknameQuery::new("   ").unwrap_err();

        assert_eq!(error, NicknameQueryError::EmptyValue);
    }
}
