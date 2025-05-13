use actix_web::{web, App, HttpResponse, HttpServer, Responder, post, get, put};
use serde::{Deserialize, Serialize};
use std::sync::{Mutex, Arc};
use uuid::Uuid;
use dotenvy::dotenv;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Todo {
    id: String,
    title: String,
    completed: bool,
}

#[derive(Debug, Serialize,  Deserialize)]
struct CreateTodoRequest {
    title: String,
}

#[derive(Debug, Serialize,  Deserialize)]
struct UpdateTodoRequest {
    title: String,
}

struct AppState {
    todos: Mutex<Vec<Todo>>,
}

#[get("/todos")]
async fn get_todos(app_state: web::Data<Arc<AppState>>) -> impl Responder {
    let todos = app_state.todos.lock().unwrap();

    HttpResponse::Ok().json(todos.clone())
}

#[get("/todos/{id}")]
async fn get_todo(app_state: web::Data<Arc<AppState>>, path: web::Path<String>) -> impl Responder {
    let todo_id = path.into_inner();
    let todos = app_state.todos.lock().unwrap();

    if let Some(todo) = todos.iter().find(|t| t.id == todo_id) {
        HttpResponse::Ok().json(todo)
    } else {
        HttpResponse::NotFound().json(format!("Todo with id {} not found", todo_id))
    }
}


#[post("/todos")]
async fn create_todo(
    app_state: web::Data<Arc<AppState>>,
    todo_req: web::Json<CreateTodoRequest>,
) -> impl Responder {
    let new_todo = Todo {
        id: Uuid::new_v4().to_string(),
        title: todo_req.title.clone(),
        completed: false,
    };

    let mut todos = app_state.todos.lock().unwrap();
    todos.push(new_todo.clone());

    HttpResponse::Created().json(new_todo)
}

#[put("/todos/{id}")]
async fn update_todo(
    app_state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
    todo_req: web::Json<UpdateTodoRequest>,
) -> impl Responder {
    let todo_id = path.into_inner();
    let mut todos = app_state.todos.lock().unwrap();
    if let Some(todo_index) = todos.iter().position(|t| t.id == todo_id) {
        todos[todo_index].title = todo_req.title.clone();
        
        HttpResponse::Ok().json(todos[todo_index].clone())
    } else {
        HttpResponse::NotFound().json(format!("Todo with id {} not found", todo_id))
    }
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().json("Hello world")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().expect(".env file not found");
    env_logger::init();
    
    let app_state = Arc::new(AppState {
        todos: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(get_todos)
            .service(get_todo)
            .service(create_todo)
            .service(hello)
            .service(update_todo)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// #[cfg(test)]
// mod tests {
//     use actix_web::http::StatusCode;
//     use actix_web::test;
//     
//     use super::*;
// 
//     #[actix_web::test]
//     async fn test_hello() {
//         let app = test::init_service(App::new().service(hello)).await;
//         let req = test::TestRequest::get().uri("/hello").to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), StatusCode::OK);
//     }
//     
//     #[actix_web::test]
//     async fn test_get_empty_todos() {
//         let app_state = Arc::new(AppState {
//             todos: Mutex::new(Vec::new()),
//         });
//         let app = test::init_service(
//             App::new().app_data(web::Data::new(app_state.clone())).service(get_todos)
//         ).await;
//         let req = test::TestRequest::get().uri("/todos").to_request();
//         let result = test::call_service(&app, req).await;
//         assert_eq!(result.status(), StatusCode::OK);
// 
//         let expected: Vec<Todo> = Vec::new();
//         let expected_json = serde_json::to_string(&expected).unwrap();
//         let actual_json = test::read_body(result).await;
//         assert_eq!(actual_json, expected_json);
//     }
// 
//     #[actix_web::test]
//     async fn test_create_todo() {
//         let app = test::init_service(App::new().service(create_todo)).await;
//         let todo_request = CreateTodoRequest {
//             title: "Test Todo".to_string(),
//         };
//         let req = test::TestRequest::post()
//             .uri("/todos")
//             .set_json(&todo_request)
//             .to_request();
//         let resp = test::call_service(&app, req).await;
//         assert_eq!(resp.status(), StatusCode::CREATED);
// 
//         let todo: Todo = test::read_body_json(resp).await;
//         assert_eq!(todo.title, "Test Todo");
//         assert_eq!(todo.completed, false);
//         assert!(!todo.id.is_empty());
//     }
// }
