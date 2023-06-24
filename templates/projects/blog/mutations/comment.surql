// $user_id: &str
// $post_id: Option<String>
// $comment_id: Option<String>
// $content: &str

LET $user = (SELECT * FROM type::thing("user", $user_id));

LET $post_or_comment = (SELECT * FROM type::thing("post", $post_id), type::thing("comment", $comment_id));

RELATE $user->comment->$post_or_comment
SET content = $content;