use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostsQueryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub number_of_comments: u16,
}

pub type PostsQuery = Vec<PostsQueryItem>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostByIdQueryComments {
    pub id: String,
    pub content: String,
    pub author: String,
    pub created_at: String,
    pub comments: Vec<PostByIdQueryComments>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostByIdQueryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub author: String,
    pub created_at: String,
    pub comments: Vec<PostByIdQueryComments>,
}

pub type PostByIdQuery = Vec<PostByIdQueryItem>;
