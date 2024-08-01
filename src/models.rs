// Generated by diesel_ext

#![allow(unused)]
#![allow(clippy::all)]

use chrono::offset::Utc;
use chrono::DateTime;
use diesel::Queryable;
use serde::Serialize;

#[derive(Queryable, Debug, Serialize)]
pub struct Note {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub created_by: i32,
    #[serde(skip_serializing)]
    pub created_on: Option<DateTime<Utc>>,
}

#[derive(Queryable, Debug, Serialize)]
pub struct User {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
}
