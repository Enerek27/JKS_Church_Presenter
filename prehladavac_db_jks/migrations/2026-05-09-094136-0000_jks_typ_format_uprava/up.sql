-- Your SQL goes here
CREATE TABLE jks (
    row_id      INTEGER PRIMARY KEY AUTOINCREMENT,
    id          INTEGER NOT NULL,
    cislo_stofy INTEGER NOT NULL,
    typ_piesne  TEXT,
    text        TEXT NOT NULL
);