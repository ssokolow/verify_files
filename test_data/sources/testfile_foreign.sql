BEGIN transaction;
CREATE TABLE "parent" (
    "id" INTEGER PRIMARY KEY autoincrement NOT NULL
);
INSERT INTO "parent" VALUES(1);
CREATE TABLE "child" (
    "id" INTEGER PRIMARY KEY autoincrement NOT NULL,
    "foreign" INTEGER REFERENCES parent (id) NOT NULL
);
INSERT INTO "child" VALUES(1,1);
COMMIT;
