mod handlers;
mod repositories;

use crate::repositories::{
    label::LabelRepositoryForDb,
    todo::{TodoRepository, TodoRepositoryForDb},
};
use axum::{
    extract::Extension,
    routing::{get, post, delete},
    Router,
};
use handlers::{
    todo::{all_todo, create_todo, delete_todo, find_todo, update_todo},
    label::{all_label, create_label, delete_label},
};
use std::net::SocketAddr;
use std::{env, sync::Arc};
use repositories::label::LabelRepository;

use dotenv::dotenv;
use hyper::header::CONTENT_TYPE;
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer, Origin};

#[tokio::main]
async fn main() {
    // logging
    let log_level = env::var("RUST_LOG").unwrap_or("into".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
    tracing::debug!("start connect database...");
    let pool = PgPool::connect(database_url)
        .await
        .expect(&format!("fail connect database, url is [{}]", database_url));

    // Routing
    let app = create_app(
        TodoRepositoryForDb::new(pool.clone()),
        LabelRepositoryForDb::new(pool.clone()),
    );
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<Todo: TodoRepository, Label: LabelRepository>(
    todo_repository: Todo,
    label_repository: Label,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<Todo>).get(all_todo::<Todo>))
        .route(
            "/todos/:id",
           get(find_todo::<Todo>)
                    .delete(delete_todo::<Todo>)
                    .patch(update_todo::<Todo>)
        )
        .route("/labels", post(create_label::<Label>).get(all_label::<Label>))
        .route("/labels/:id", delete(delete_label::<Label>))
        .layer(Extension(Arc::new(todo_repository)))
        .layer(Extension(Arc::new(label_repository)))
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::exact("http://localhost:3001".parse().unwrap()))
                .allow_methods(Any)
                .allow_headers(vec![CONTENT_TYPE]),
        )
}

async fn root() -> &'static str {
    "Hello, world!"
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repositories::{
        todo::{CreateTodo, Todo, test_utils::TodoRepositoryForMemory},
        label::{test_utils::LabelRepositoryForMemory}
    };
    use axum::response::Response;
    use axum::{body::Body, http::{header, Method, Request, StatusCode}};
    use tower::ServiceExt;
    use crate::repositories::label::Label;

    fn build_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }

    fn build_todo_req_with_empty(method: Method, path: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_todo(res: Response) -> Todo {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();

        let todo: Todo = serde_json::from_str(&body).expect(&format!("cannot convert Todo instance. Body is {}", body));
        todo
    }

    async fn res_to_label(res: Response) -> Label {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();

        let label: Label = serde_json::from_str(&body).expect(&format!("cannot convert Todo instance. Body is {}", body));
        label
    }

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(TodoRepositoryForMemory::new(), LabelRepositoryForMemory::new()).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();

        assert_eq!(body, "Hello, world!");
    }

    #[tokio::test]
    async fn should_created_todo() {
        let expected = Todo::new(1, "should_return_created_todo".to_string());

        let req = build_req_with_json(
            "/todos",
            Method::POST,
            r#"{ "text": "should_return_created_todo" }"#.to_string()
        );
        let res = create_app(TodoRepositoryForMemory::new(), LabelRepositoryForMemory::new()).oneshot(req).await.unwrap();

        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_find_todo() {
        let expected = Todo::new(1, "should_find_todo".to_string());

        let repository = TodoRepositoryForMemory::new();
        repository
            .create(CreateTodo::new("should_find_todo".to_string()))
            .await
            .expect("failed create todo");

        let req = build_todo_req_with_empty(
            Method::GET,
            "/todos/1",
        );
        let res = create_app(repository, LabelRepositoryForMemory::new()).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_get_all_todo() {
        let expected = Todo::new(1, "should_get_all_todo".to_string());

        let todo_repository = TodoRepositoryForMemory::new();
        todo_repository
            .create(CreateTodo::new("should_get_all_todo".to_string()))
            .await
            .expect("failed create todo");
        let req = build_todo_req_with_empty(
            Method::GET,
            "/todos",
        );
        let res = create_app(todo_repository, LabelRepositoryForMemory::new()).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();

        let todo: Vec<Todo> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Todo instance. Body is {}", body));

        assert_eq!(vec![expected], todo);
    }

    #[tokio::test]
    async fn should_updated_todo() {
        let expected = Todo::new(1, "should_updated_todo".to_string());

        let todo_repository = TodoRepositoryForMemory::new();
        todo_repository
            .create(CreateTodo::new("before_update_todo".to_string()))
            .await
            .expect("failed create todo");
        let req = build_req_with_json(
            "/todos/1",
            Method::PATCH,
            r#"{
    "id": 1,
    "text": "should_updated_todo",
    "completed": false
}"#
                .to_string(),
        );
        let res = create_app(todo_repository, LabelRepositoryForMemory::new()).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_deleted_todo() {
        let todo_repository = TodoRepositoryForMemory::new();
        todo_repository
            .create(CreateTodo::new("should_deleted_todo".to_string()))
            .await
            .expect("failed create todo");
        let req = build_todo_req_with_empty(Method::DELETE, "/todos/1");
        let res = create_app(todo_repository, LabelRepositoryForMemory::new()).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }

    #[tokio::test]
    async fn should_created_label() {
        let expected = Label::new(1, "should_created_label".to_string());

        let req = build_req_with_json(
            "/labels",
            Method::POST,
            r#"{ "name": "should_created_label" }"#.to_string()
        );
        let res = create_app(TodoRepositoryForMemory::new(), LabelRepositoryForMemory::new()).oneshot(req).await.unwrap();
        let label = res_to_label(res).await;
        assert_eq!(expected, label);
    }

    #[tokio::test]
    async fn should_get_all_label() {
        let expected = Label::new(1, "should_get_all_label".to_string());

        let label_repository = LabelRepositoryForMemory::new();
        label_repository
            .create("should_get_all_label".to_string())
            .await
            .expect("failed create todo");
        let req = build_todo_req_with_empty(
            Method::GET,
            "/labels",
        );
        let res = create_app(TodoRepositoryForMemory::new(), label_repository).oneshot(req).await.unwrap();

        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();

        let labels: Vec<Label> = serde_json::from_str(&body)
            .expect(&format!("cannot convert Label instance. Body is {}", body));

        assert_eq!(vec![expected], labels);
    }

    #[tokio::test]
    async fn should_deleted_label() {
        let label_repository = LabelRepositoryForMemory::new();
        label_repository
            .create("delete_label".to_string())
            .await
            .expect("failed create Label");
        let req = build_todo_req_with_empty(Method::DELETE, "/labels/1");
        let res = create_app(TodoRepositoryForMemory::new(), label_repository).oneshot(req).await.unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }
}
