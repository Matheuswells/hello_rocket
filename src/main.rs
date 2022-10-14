#![feature(proc_macro_hygiene, decl_macro)]

use serde::{Serialize};
use rocket_contrib::json::Json;
use rusqlite::Connection;
#[macro_use] extern crate rocket;

#[derive(Serialize)]
struct TodoList  {
    items: Vec<TodoItem>

}
#[derive(Serialize)]
struct TodoItem {
    id: i64,
    item: String,
}
#[derive(Serialize)]
struct StatusMessage {
    message: String,
}

#[derive(Serialize)]
struct TodoItemData {
    description: String,
}
#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/todo")]
fn get_todo() -> Result<Json<TodoList>, String> {
    let db_connection =  match Connection::open("data.sqlite") {
        Ok(conn) => conn,
        Err(_) => return Err(format!("Failed to connect to database")),
    };

    let mut stmt = match db_connection.prepare("select id, item from todo_list;") {
        Ok(stmt) => stmt,
        Err(e) => return Err(format!("Failed to prepare statement > {}", e)),
    };

    let results = stmt.query_map([], |row| {
        Ok(TodoItem{
            id: row.get(0)?,
            item: row.get(1)?,
        })
    });

    match results {
        Ok(rows) => {
            let collection: rusqlite::Result<Vec<_>> = rows.collect();

            match collection {
                Ok(items) => Ok(Json(TodoList{items})),
                Err(_) => Err(format!("Failed to collect results")),
            }
        }
        Err(_) => return Err(format!("Failed to query database")),
    }
}

#[post("/todo", format = "json", data = "<item>")]
fn add_todo_item(item: Json<String>) -> Result<Json<StatusMessage>, String> {
    let db_connection =  match Connection::open("data.sqlite") {
        Ok(conn) => conn,
        Err(_) => return Err(format!("Failed to connect to database")),
    };

    let mut stmt = match db_connection
    .prepare("insert into todo_list (id, item) values (null, $1);") {
        Ok(stmt) => stmt,
        Err(e) => return Err(format!("Failed to prepare statement > {}", e)),
    };

    let results = stmt.execute(&[&item.0]);

    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage { message : format!("{} rows affected", rows_affected) })),
        Err(_) => Err(format!("Failed to fetch todo items results")),
    }
}

#[delete("/todo/<id>")]
fn delete_todo_item(id: i64) -> Result<Json<StatusMessage>, String> {
    let db_connection =  match Connection::open("data.sqlite") {
        Ok(conn) => conn,
        Err(_) => return Err(format!("Failed to connect to database")),
    };

    let mut stmt = match db_connection
    .prepare("delete from todo_list where id = $1;") {
        Ok(stmt) => stmt,
        Err(e) => return Err(format!("Failed to prepare statement > {}", e)),
    };

    let results = stmt.execute(&[&id]);

    match results {
        Ok(rows_affected) => Ok(Json(StatusMessage { message : format!("{} rows affected", rows_affected) })),
        Err(_) => Err(format!("Failed to delete todo items")),
    }
}

fn main () {
    {
        let db_connection = Connection::open("data.sqlite").unwrap();
        db_connection
            .execute(
                "create table if not exists todo_list (
                    id integer primary key,
                    item varchar(64) not null
                );",
            []).unwrap();
    
    }

   rocket::ignite().mount(
    "/",
     routes![
        index,
        get_todo, 
        add_todo_item, 
        delete_todo_item
        ]).launch();
}
