use super::insertables::NewNote;
use super::messages::*;
use crate::models::Note;
use crate::schema::notes::{dsl::*, id as note_id};
use crate::utils::db::DbActor;
use actix::Handler;
use diesel::associations::HasTable;
use diesel::prelude::*;

impl Handler<FetchNotes> for DbActor {
    type Result = QueryResult<Vec<Note>>;

    fn handle(&mut self, msg: FetchNotes, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Fetch Notes: Unable to establish connection");

        let mut query = notes::table().into_boxed();

        if let Some(search_term) = msg.search {
            let search_pattern: String = format!("%{}%", search_term);

            query = query.filter(
                title
                    .ilike(search_pattern.clone())
                    .or(content.ilike(search_pattern)),
            );
        }

        if let Some(sort_field) = msg.sort_field {
            match sort_field.as_str() {
                "title" => {
                    if let Some(sort_order) = msg.sort_order {
                        match sort_order.as_str() {
                            "asc" => query = query.order(title.asc()),
                            "desc" => query = query.order(title.desc()),
                            _ => {}
                        }
                    }
                }
                "content" => {
                    if let Some(sort_order) = msg.sort_order {
                        match sort_order.as_str() {
                            "asc" => query = query.order(content.asc()),
                            "desc" => query = query.order(content.desc()),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(limit) = msg.limit {
            query = query.limit(limit);
        }

        if let Some(offset) = msg.offset {
            query = query.offset(offset);
        }

        query.get_results::<Note>(&mut connection)
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

        let new_note: NewNote = NewNote {
            title: msg.title,
            content: msg.content,
            created_by: msg.created_by,
            created_on: msg.created_on,
            updated_on: msg.updated_on,
        };

        diesel::insert_into(notes)
            .values(new_note)
            .returning((
                note_id,
                title,
                content,
                created_by,
                created_on.nullable(),
                updated_on.nullable(),
            ))
            .get_result::<Note>(&mut connection)
    }
}

impl Handler<UpdateNote> for DbActor {
    type Result = QueryResult<Note>;

    fn handle(&mut self, msg: UpdateNote, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Update Note: Unable to establish connection");

        diesel::update(notes.find(msg.id))
            .set((
                title.eq(msg.title),
                content.eq(msg.content),
                created_by.eq(msg.created_by),
                updated_on.eq(msg.updated_on),
            ))
            .get_result::<Note>(&mut connection)
    }
}

impl Handler<DeleteNote> for DbActor {
    type Result = Result<usize, diesel::result::Error>;

    fn handle(&mut self, msg: DeleteNote, _ctx: &mut Self::Context) -> Self::Result {
        let mut connection = self
            .0
            .get()
            .expect("Delete Note: Unable to establish connection");

        diesel::delete(notes.find(msg.note_id)).execute(&mut connection)
    }
}
