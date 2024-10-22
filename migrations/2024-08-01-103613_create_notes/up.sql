-- Your SQL goes here
CREATE TABLE
  users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(30) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(80) NOT NULL UNIQUE,
    opt_verified BOOLEAN DEFAULT FALSE,
    opt_enabled BOOLEAN DEFAULT FALSE,
    opt_base32 VARCHAR(100) DEFAULT NULL,
    opt_auth_url VARCHAR(255) DEFAULT NULL,
    role VARCHAR(10) DEFAULT 'user' NOT NULL
  );

CREATE TABLE
  notes (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    image_url VARCHAR(255) DEFAULT NULL,
    active BOOLEAN DEFAULT FALSE,
    created_by INT4 NOT NULL,
    created_on TIMESTAMPTZ,
    updated_on TIMESTAMPTZ,
    FOREIGN KEY (created_by) REFERENCES users (id)
  );