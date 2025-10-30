use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use shared::database::Database;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(online_check).service(
            web::resource("/ticket/{path}")
                .route(web::get().to(get_ticket))
                .route(web::patch().to(patch_ticket_seen))
                .route(web::delete().to(delete_ticket_seen)),
        )
    })
    .bind(("0.0.0.0", 6767))?
    .run()
    .await
}

#[get("/")]
async fn online_check() -> impl Responder {
    HttpResponse::Ok().body("Online")
}

async fn get_ticket(path: web::Path<String>) -> impl Responder {
    match Database::get_by_hash(&path) {
        Some(hash_struct) => HttpResponse::Ok().json(hash_struct),
        None => HttpResponse::NotFound().finish(),
    }
}

async fn patch_ticket_seen(path: web::Path<String>) -> impl Responder {
    match Database::mark_ticket_seen(&path) {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(err) => match err.to_string().as_str() {
            "Ticket hash not found" => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().body(err.to_string()),
        },
    }
}

async fn delete_ticket_seen(path: web::Path<String>) -> impl Responder {
    match Database::unmark_ticket_seen(&path) {
        Ok(()) => HttpResponse::Ok().finish(),
        Err(err) => match err.to_string().as_str() {
            "Ticket hash not found" => HttpResponse::NotFound().finish(),
            _ => HttpResponse::InternalServerError().body(err.to_string()),
        },
    }
}
