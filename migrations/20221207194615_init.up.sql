CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS Users (
    username text NOT NULL PRIMARY KEY,
    email text NOT NULL UNIQUE,
    password text NOT NULL,
    bio text NULL,
    image text NULL
);

CREATE TABLE IF NOT EXISTS Follows (
    follower text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    influencer text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (follower, influencer)
);

CREATE TABLE IF NOT EXISTS Articles (
    slug text NOT NULL PRIMARY KEY,
    author text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    title text NOT NULL,
    description text NOT NULL,
    body text NOT NULL,
    created_at TIMESTAMPTZ NOT NULL default NOW(),
    updated_at TIMESTAMPTZ NOT NULL default NOW()
);

CREATE TABLE IF NOT EXISTS ArticleTags (
    article text NOT NULL REFERENCES Articles(slug) ON DELETE CASCADE ON UPDATE CASCADE,
    tag text NOT NULL,
    PRIMARY KEY (article, tag)
);

CREATE INDEX IF NOT EXISTS tags ON ArticleTags (tag);

CREATE TABLE IF NOT EXISTS FavArticles (
    article text NOT NULL REFERENCES Articles(slug) ON DELETE CASCADE ON UPDATE CASCADE,
    username text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    PRIMARY KEY (article, username)
);

CREATE TABLE IF NOT EXISTS Comments (
    id int PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    article text NOT NULL REFERENCES Articles(slug) ON DELETE CASCADE ON UPDATE CASCADE,
    username text NOT NULL REFERENCES Users(username) ON DELETE CASCADE ON UPDATE CASCADE,
    body text NOT NULL,
    created_at TIMESTAMPTZ NOT NULL default NOW()
);
