use super::insertables::NewNote;
use super::messages::{CreateNote, FetchUser, FetchUserNotes};
use crate::models::{Note, User};
use crate::schema::notes::{dsl::*, id as note_id};
use crate::schema::users::dsl::*;
use crate::utils::db::DbActor;
use actix::Handler;
use diesel::prelude::*;

impl Handler<FetchUser> for DbActor {
    type Result = QueryResult<Vec<User>>;

    fn handle(&mut self, _msg: FetchUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Fetch User: Unable to establish connection");

        users.get_results::<User>(&mut connection)
    }
}

impl Handler<FetchUserNotes> for DbActor {
    type Result = QueryResult<Vec<Note>>;

    fn handle(&mut self, msg: FetchUserNotes, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Fetch User Notes: Unable to establish connection");

        notes
            .filter(created_by.eq(msg.user_id))
            .get_results::<Note>(&mut connection)
    }
}

impl Handler<CreateNote> for DbActor {
    type Result = QueryResult<Note>;

    fn handle(&mut self, msg: CreateNote, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Create User Note: Unable to establish connection");

        let new_article: NewNote = NewNote {
            title: msg.title,
            content: msg.content,
            created_by: msg.created_by,
        };

        diesel::insert_into(notes)
            .values(new_article)
            .returning((note_id, title, content, created_by, created_on.nullable()))
            .get_result::<Note>(&mut connection)
    }
}
