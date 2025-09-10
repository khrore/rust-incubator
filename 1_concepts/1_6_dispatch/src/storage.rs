use crate::base::{Storage, User};
use std::borrow::Cow;
use std::{collections::HashMap, hash::Hash};

struct UserStorage<K>(HashMap<K, User>);

impl<K> Storage<K, User> for UserStorage<K>
where
    K: Hash + Eq,
{
    fn set(&mut self, key: K, val: User) {
        self.0.insert(key, val);
    }
    fn get(&self, key: &K) -> Option<&User> {
        self.0.get(key)
    }
    fn remove(&mut self, key: &K) -> Option<User> {
        self.0.remove(key)
    }
}

#[test]
fn disp_test() {
    let mut stor = UserStorage::<String>(HashMap::new());

    let name1 = "Miku".to_string();
    let user1 = User {
        id: 0,
        email: Cow::Owned("kal@mail.ru".into()),
        activated: true,
    };

    let name2 = "Oleg".to_string();
    let user2 = User {
        id: 1,
        email: Cow::Owned("zhizn@true.gov".into()),
        activated: false,
    };

    stor.set(name1, user1);
    stor.set(name2, user2);

    assert_eq!(stor.get(&"Oleg".to_string()).unwrap().id, 1);
    assert_eq!(stor.get(&"Miku".to_string()).unwrap().id, 0);

    assert_eq!(stor.remove(&"Lokh".to_string()), None);
    assert_eq!(stor.remove(&"Oleg".to_string()).unwrap().id, 1);
    assert_eq!(stor.get(&"Oleg".to_string()), None);
}
