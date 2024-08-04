# Rust Note API

A simple RESTful API for managing notes, secured with JWT authentication, built with Rust using Actix Web, Diesel, and PostgreSQL.

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

### Endpoint: /user/register

```json
{
  "username": "dipeshpaudel",
  "email": "dipeshpaudel@gmail.com",
  "password": "dipeshpaudel"
}
```

### Endpoint: /user/login

```json
{
  "email": "dipeshpaudel@gmail.com",
  "password": "dipeshpaudel"
}
```

### Endpoint: /secure/api/user/note

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

### Endpoint: /secure/api/user/notes

## PATCH Method:

### Endpoint: /secure/api/user/note/update/{note_id}

```json
{
  "title": "Mobile Phone"
}
```

## DELETE Method:

### Endpoint: /secure/api/user/note/delete/{note_id}
