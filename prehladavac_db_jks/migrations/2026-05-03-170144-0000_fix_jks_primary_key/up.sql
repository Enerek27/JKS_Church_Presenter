-- Your SQL goes here
DROP TABLE IF EXISTS jks;

CREATE TABLE jks (
    row_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    id          INTEGER NOT NULL,
    cislo_stofy INTEGER NOT NULL,
    text        TEXT NOT NULL
);