DROP TABLE MrAdultRepositories;

CREATE TABLE IF NOT EXISTS MrAdultRepositories (
   id BIGINT UNIQUE NOT NULL,
   name TEXT NOT NULL UNIQUE,
   alphanumeric_name TEXT NOT NULL,
   url TEXT NOT NULL,
   html_url TEXT NOT NULL,
   description TEXT NOT NULL,
   updated_at TIMESTAMPTZ NOT NULL,
   readme TEXT
);

CREATE INDEX idx_repo_name ON MrAdultRepositories(alphanumeric_name);