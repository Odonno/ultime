// $post_id: Option<String>
// $comment_id: Option<String>
// $content: String

LET $user = (SELECT * FROM $auth);

LET $post_or_comment = (SELECT * FROM type::thing("post", $post_id), type::thing("comment", $comment_id));

RELATE $user->comment->$post_or_comment
SET content = $content;