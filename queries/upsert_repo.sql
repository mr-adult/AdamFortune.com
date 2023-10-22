INSERT INTO MrAdultRepositories( id, name, url, html_url, description, updated_at, readme ) 
    VALUES ( $1, $2, $3, $4, $5, $6, $7 ) 
    ON CONFLICT (id) DO
    UPDATE SET 
        name = EXCLUDED.name,
        url = EXCLUDED.url,
        html_url = EXCLUDED.html_url,
        description = EXCLUDED.description,
        updated_at = EXCLUDED.updated_at,
        readme = EXCLUDED.readme;