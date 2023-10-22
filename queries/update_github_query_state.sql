UPDATE GitHubQueryState SET last_queried = 
    CASE WHEN (last_queried + INTERVAL '1 HOUR') > NOW() 
    THEN last_queried 
    ELSE NOW() 
    END;