use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommentMutationItem {
    pub id: String,
    pub content: String,
}

pub type CommentMutation = Vec<CommentMutationItem>;
