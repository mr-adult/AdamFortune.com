DROP TABLE BlogPosts;

CREATE TABLE IF NOT EXISTS BlogPosts (
   id SERIAL PRIMARY KEY,
   name TEXT NOT NULL,
   alphanumeric_name TEXT NOT NULL,
   description TEXT NOT NULL,
   sha TEXT NOT NULL UNIQUE,
   content TEXT NOT NULL,
   CONSTRAINT alphanumeric_name_unique UNIQUE (alphanumeric_name)
);