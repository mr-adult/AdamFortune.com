CREATE TABLE IF NOT EXISTS GitHubQueryState (
    id SERIAL,
    last_queried TIMESTAMPTZ NOT NULL
);

-- Start at default Unix TimeStamp of 1970-01-01
INSERT INTO GitHubQueryState (id, last_queried) VALUES (1, TIMESTAMPTZ '1970-01-01 00:00:00 UTC');



CREATE TABLE IF NOT EXISTS MrAdultRepositories (
   id BIGINT UNIQUE NOT NULL,
   name TEXT NOT NULL UNIQUE,
   url TEXT NOT NULL,
   html_url TEXT NOT NULL,
   description TEXT NOT NULL,
   updated_at TIMESTAMPTZ NOT NULL,
   readme TEXT
);

CREATE INDEX idx_repo_name ON MrAdultRepositories(name);



CREATE TABLE IF NOT EXISTS BlogPosts (
   id SERIAL PRIMARY KEY,
   name TEXT NOT NULL,
   description TEXT NOT NULL,
   sha TEXT NOT NULL UNIQUE,
   content TEXT NOT NULL
);

CREATE INDEX idx_blog_post_name ON BlogPosts(name);