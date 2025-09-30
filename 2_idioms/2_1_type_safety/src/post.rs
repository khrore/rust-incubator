use std::marker::PhantomData;

mod userdata {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Id(pub u64);
}

mod postdata {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Id(pub u64);

    #[derive(Clone, Debug, PartialEq)]
    pub struct Title(pub String);

    #[derive(Clone, Debug, PartialEq)]
    pub struct Body(pub String);
}

#[derive(Clone)]
struct PostData {
    id: postdata::Id,
    user_id: userdata::Id,
    title: postdata::Title,
    body: postdata::Body,
}

#[derive(Clone)]
pub struct Post<S: PostState> {
    data: PostData,
    _state: PhantomData<S>,
}

pub enum New {}
pub enum Unmoderated {}
pub enum Published {}
pub enum Deleted {}

pub trait PostState {}
impl PostState for New {}
impl PostState for Unmoderated {}
impl PostState for Published {}
impl PostState for Deleted {}

impl<S> Post<S>
where
    S: PostState,
{
    pub fn new() -> Post<New> {
        Post {
            data: PostData::default(),
            _state: PhantomData,
        }
    }
}

impl Post<New> {
    pub fn publish(self) -> Post<Unmoderated> {
        Post {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl Post<Unmoderated> {
    pub fn allow(self) -> Post<Published> {
        Post {
            data: self.data,
            _state: PhantomData,
        }
    }

    pub fn deny(self) -> Post<Deleted> {
        Post {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl Post<Published> {
    pub fn delete(self) -> Post<Deleted> {
        Post {
            data: self.data,
            _state: PhantomData,
        }
    }
}

impl PostData {
    pub fn new(
        id: postdata::Id,
        user_id: userdata::Id,
        title: postdata::Title,
        body: postdata::Body,
    ) -> Self {
        Self {
            id,
            user_id,
            title,
            body,
        }
    }
}

impl Default for PostData {
    fn default() -> Self {
        Self::new(
            postdata::Id(0),
            userdata::Id(0),
            postdata::Title("New".to_owned()),
            postdata::Body("Some body once told me".to_owned()),
        )
    }
}
