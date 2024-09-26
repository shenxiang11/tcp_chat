use std::fmt::{Display, Formatter};

pub(crate) enum Message {
    UserJoin(String),
    UserLeave(String),
    Chat {
        username: String,
        message: String,
    },
}

impl Message {
    pub(crate) fn user_join(username: impl Into<String>) -> Self {
        let content = format!("System: {} has joined the chat", username.into());
        Self::UserJoin(content)
    }

    pub(crate) fn user_leave(username: impl Into<String>) -> Self {
        let content = format!("System: {} has left the chat", username.into());
        Self::UserLeave(content)
    }

    pub(crate) fn chat(username: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Chat {
            username: username.into(),
            message: message.into(),
        }
    }
}

impl Display for Message {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UserJoin(username) => write!(f, "{}", username),
            Self::UserLeave(username) => write!(f, "{}", username),
            Self::Chat { username, message } => write!(f, "{}: {}", username, message),
        }
    }
}
