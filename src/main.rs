use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use askama::Template;
use std::sync::{Mutex, Arc};
use std::time::Instant;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    title: String,
    tasks: Vec<Task>,
}

#[derive(Clone)]
struct Task {
    description: String,
    time_spent: u64, // time in seconds
    running: bool,
    start_time: Option<Instant>,
}

struct AppState {
    tasks: Mutex<Vec<Task>>,
}

async fn index(data: web::Data<Arc<AppState>>) -> impl Responder {
    let tasks = data.tasks.lock().unwrap().clone();
    let template = IndexTemplate {
        title: "Time Tracker".to_string(),
        tasks,
    };

    HttpResponse::Ok().content_type("text/html").body(template.render().unwrap())
}

#[derive(serde::Deserialize)]
struct TaskForm {
    description: String,
}

async fn add_task(data: web::Data<Arc<AppState>>, form: web::Form<TaskForm>) -> impl Responder {
    let mut tasks = data.tasks.lock().unwrap();
    tasks.push(Task {
        description: form.description.clone(),
        time_spent: 0,
        running: true,  // Start tracking time immediately
        start_time: Some(Instant::now()),
    });

    HttpResponse::SeeOther().append_header(("Location", "/")).finish()
}

async fn start_stop_task(data: web::Data<Arc<AppState>>, task_id: web::Path<usize>) -> impl Responder {
    let mut tasks = data.tasks.lock().unwrap();
    let task = &mut tasks[*task_id];

    if task.running {
        // Stop the task
        if let Some(start_time) = task.start_time {
            task.time_spent += start_time.elapsed().as_secs();
        }
        task.running = false;
        task.start_time = None;
    } else {
        // Start the task
        task.running = true;
        task.start_time = Some(Instant::now());
    }

    HttpResponse::SeeOther().append_header(("Location", "/")).finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let app_state = Arc::new(AppState {
        tasks: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .route("/", web::get().to(index))
            .route("/add_task", web::post().to(add_task))
            .route("/start_stop/{id}", web::post().to(start_stop_task))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
