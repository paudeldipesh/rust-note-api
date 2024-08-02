# Rust Note API

A simple RESTful API for managing notes, built with Rust using Actix Web, Diesel, and PostgreSQL.

## Features

- Test routes (home/index/hello)

- Create a new user

- Create a new note

- Read an existing note by ID

- Update an existing note

- Delete a note by ID

- List all notes

- List all users

## Requirements

- Rust (latest stable version)

- PostgreSQL

- Cargo package manager

## Setup

### Clone the repository

```bash

git  clone  https://github.com/paudeldipesh/rust-note-api.git

cd  rust-note-api

```

## POST Methods:

### Endpoint: /api/user

```json
{
  "first_name": "Dipesh",

  "last_name": "Paudel",

  "username": "dipeshpaudel",

  "email": "dipesh@paudel.com"
}
```

### Endpoint: /api/user/{user_id}/note

```json
{
  "title": "Computer",

  "content": "An electronic device."
}
```

## GET Methods:

### Endpoint: /

### Endpoint: /test/hello-world

### Endpoint: /test/Dipesh

### Endpoint: /api/users

### Endpoint: /api/notes

### Endpoint: /api/user/{user_id}/notes

## PATCH Methods:

### Endpoint: /api/user/{user_id}/note/{note_id}

```json
{
  "title": "Mobile Phone"
}
```

## DELETE Methods:

### Endpoint: /api/user/{random_user_id}/note/{note_id}
