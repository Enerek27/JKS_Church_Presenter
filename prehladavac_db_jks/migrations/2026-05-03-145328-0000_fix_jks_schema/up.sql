-- Your SQL goes here
-- up.sql
DROP TABLE IF EXISTS jks;

CREATE TABLE IF NOT EXISTS jks (
    id              INTEGER PRIMARY KEY NOT NULL,
    cislo_stofy     INTEGER NOT NULL,
    text            TEXT NOT NULL
);


