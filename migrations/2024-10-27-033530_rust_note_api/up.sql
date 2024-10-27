-- Your SQL goes here
-- Your SQL goes here
CREATE TABLE
  users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(30) NOT NULL UNIQUE,
    email VARCHAR(100) NOT NULL UNIQUE,
    password VARCHAR(80) NOT NULL UNIQUE,
    otp_verified BOOLEAN DEFAULT FALSE,
    otp_enabled BOOLEAN DEFAULT FALSE,
    otp_base32 VARCHAR(100) DEFAULT NULL,
    otp_auth_url VARCHAR(255) DEFAULT NULL,
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