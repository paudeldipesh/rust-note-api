-- Your SQL goes here
CREATE TABLE
  users (
    id SERIAL PRIMARY KEY,
    first_name VARCHAR(50) NOT NULL,
    last_name VARCHAR(50) NOT NULL,
    username VARCHAR(30) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE
  );

CREATE TABLE
  notes (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    created_by INT4 NOT NULL,
    created_on TIMESTAMPTZ,
    FOREIGN KEY (created_by) REFERENCES users (id)
  );