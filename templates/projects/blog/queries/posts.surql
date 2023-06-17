SELECT 
    meta::id(id) AS id,
    title,
    content,
    status,
    count(<-comment) + count(<-comment<-comment) AS number_of_comments
FROM post;