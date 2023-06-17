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
pub struct PostQueryComments {
    pub id: String,
    pub content: String,
    pub author: String,
    pub created_at: String,
    pub comments: Vec<PostQueryComments>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostQueryItem {
    pub id: String,
    pub title: String,
    pub content: String,
    pub status: String,
    pub author: String,
    pub created_at: String,
    pub comments: Vec<PostQueryComments>,
}

pub type PostQuery = Vec<PostQueryItem>;
