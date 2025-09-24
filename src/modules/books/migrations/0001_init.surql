-- Sample migration for the Books module
DEFINE TABLE book SCHEMAFULL;
DEFINE FIELD title  ON book TYPE string ASSERT $value != "";
DEFINE FIELD author ON book TYPE string ASSERT $value != "";
