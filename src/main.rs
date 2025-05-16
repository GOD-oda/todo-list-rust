use actix_web::{web, App, HttpResponse, HttpServer, Responder, post, get, put, delete};
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

#[get("")]
async fn get_todos(app_state: web::Data<Arc<AppState>>) -> impl Responder {
    let todos = app_state.todos.lock().unwrap();

    HttpResponse::Ok().json(todos.clone())
}

#[get("/{id}")]
async fn get_todo(app_state: web::Data<Arc<AppState>>, path: web::Path<String>) -> impl Responder {
    let todo_id = path.into_inner();
    let todos = app_state.todos.lock().unwrap();

    if let Some(todo) = todos.iter().find(|t| t.id == todo_id) {
        HttpResponse::Ok().json(todo)
    } else {
        HttpResponse::NotFound().json(format!("Todo with id {} not found", todo_id))
    }
}


#[post("")]
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

#[put("/{id}")]
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

#[delete("/{id}")]
async fn delete_todo(
    app_state: web::Data<Arc<AppState>>,
    path: web::Path<String>,
) -> impl Responder {
    let todo_id = path.into_inner();
    let mut todos = app_state.todos.lock().unwrap();
    if let Some(todo_index) = todos.iter().position(|t| t.id == todo_id) {
        todos.remove(todo_index);

        HttpResponse::NoContent().json("")
    } else {
        HttpResponse::NotFound().json(format!("Todo with id {} not found", todo_id))
    }
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
            .service(
                web::scope("/todos")
                    .service(get_todos)
                    .service(get_todo)
                    .service(create_todo)
                    .service(update_todo)
                    .service(delete_todo)
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;
    use super::*;

    #[actix_web::test]
    async fn test_get_empty_todos() {
        let app_state = Arc::new(AppState {
            todos: Mutex::new(Vec::new()),
        });
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(web::scope("/todos").service(get_todos))
        ).await;
        let req = test::TestRequest::get().uri("/todos").to_request();
        let result = test::call_service(&app, req).await;
        assert_eq!(result.status(), StatusCode::OK);

        let expected: Vec<Todo> = Vec::new();
        let expected_json = serde_json::to_string(&expected).unwrap();
        let actual_json = test::read_body(result).await;
        assert_eq!(actual_json, expected_json);
    }

    #[actix_web::test]
    async fn test_get_todos() {
        let mut v = Vec::new();
        v.push(Todo {
            id: Uuid::new_v4().to_string(),
            title: "title".to_string(),
            completed: false,
        });
        let app_state = Arc::new(AppState {
            todos: Mutex::new(v),
        });
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(web::scope("/todos").service(get_todos))
        ).await;
        let req = test::TestRequest::get().uri("/todos").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let todos: Vec<Todo> = test::read_body_json(resp).await;
        assert_eq!(todos.len(), 1);
    }

    #[actix_web::test]
    async fn test_create_todo() {
        let app_state = Arc::new(AppState {
            todos: Mutex::new(Vec::new()),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(web::scope("/todos").service(create_todo))
        ).await;

        let todo_request = CreateTodoRequest {
            title: "test".to_string(),
        };
        let req = test::TestRequest::post()
            .uri("/todos")
            .set_json(&todo_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let todo: Todo = test::read_body_json(resp).await;
        assert_eq!(todo.title, "test");
        assert_eq!(todo.completed, false);
        assert!(!todo.id.is_empty());
    }

    #[actix_web::test]
    async fn test_update_todo() {
        let id = Uuid::new_v4().to_string();
        let old_todo = Todo {
            id,
            title: "title".to_string(),
            completed: false,
        }; 
        let mut v = Vec::new();
        v.push(old_todo.clone());
        let app_state = Arc::new(AppState {
            todos: Mutex::new(v),
        });
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .service(web::scope("/todos").service(update_todo))
        ).await;
        let todo_request = UpdateTodoRequest {
            title: "new".to_string(),
        };
        let req = test::TestRequest::put()
            .uri(&format!("/todos/{}", old_todo.id))
            .set_json(&todo_request)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let todo: Todo = test::read_body_json(resp).await;
        assert_eq!(todo.title, "new");
    }
}
