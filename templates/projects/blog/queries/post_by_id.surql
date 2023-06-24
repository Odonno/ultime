// $post_id: &str

SELECT 
    meta::id(id) AS id,
    title,
    content,
    status,
    created_at,
    author.username AS author,
    (
        SELECT 
            meta::id(id) AS id,
            content,
            created_at,
            in.username AS author,
            (
                SELECT 
                    meta::id(id) AS id,
                    content,
                    created_at,
                    in.username AS author,
                    [] AS comments
                FROM <-comment
            ) AS comments
        FROM <-comment
    ) AS comments
FROM type::thing("post", $post_id);