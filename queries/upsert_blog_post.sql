INSERT INTO BlogPosts( name, description, sha, content ) 
    VALUES ( $1, $2, $3, $4 ) 
    ON CONFLICT (id) DO
    UPDATE SET 
        name = EXCLUDED.name,
        description = EXCLUDED.description,
        sha = EXCLUDED.sha,
        content = EXCLUDED.content;