use crate::Result;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub mod yandex;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    role: String,
    text: String,
}

pub trait TextAdapter<'a> {
    fn messages(&mut self, message: Message);
    fn assistant<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
        Self: Sized,
    {
        self.messages(Message {
            role: "assistant".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }

    fn user<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
        Self: Sized,
    {
        self.messages(Message {
            role: "user".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }

    fn system<S>(mut self, message: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
        Self: Sized,
    {
        self.messages(Message {
            role: "system".into(),
            text: message.into().to_string(),
        });
        Ok(self)
    }
}
