use serde::{Serialize, Deserialize};
use surrealdb::sql::Thing;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostsQueryItem {
    pub id: Thing,
    pub title: String,
    pub content: String,
    pub status: String,
    pub number_of_comments: u16,
}

pub type PostsQuery = Vec<PostsQueryItem>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostQueryComments {
    pub id: Thing,
    pub content: String,
    pub author: String,
    pub created_at: String,
    pub comments: Vec<PostQueryComments>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PostQueryItem {
    pub id: Thing,
    pub title: String,
    pub content: String,
    pub status: String,
    pub author: String,
    pub created_at: String,
    pub comments: Vec<PostQueryComments>,
}

pub type PostQuery = Vec<PostQueryItem>;