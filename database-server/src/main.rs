mod listener;
mod broadcaster;

use actix_web::{App, HttpResponse, HttpServer, Responder, get, web};
use shared::database::Database;
use std::thread;

/// port where the database is served
const DATAB_PORT: u16 = 6767;
/// port where we listen and reply to broadcasts
const REPLY_PORT: u16 = 6768;
/// port where we ourselves brodcast
const BROAD_PORT: u16 = 6769;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    thread::spawn(|| {
        listener::reply_to_broadcasts();
    });
    thread::spawn(|| {
        broadcaster::start_broadcast();
    });

    println!("Starting HTTP server");
    HttpServer::new(|| {
        App::new()
            .service(online_check)
            .service(
                web::resource("/ticket/{path}")
                    .route(web::get().to(get_ticket))
                    .route(web::patch().to(patch_ticket_seen))
                    .route(web::delete().to(delete_ticket_seen)),
            )
            .service(web::resource("/debug/panic").route(web::get().to(debug_panic)))
    })
    .bind(("0.0.0.0", DATAB_PORT))?
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

#[allow(unreachable_code)]
#[cfg(debug_assertions)]
async fn debug_panic() -> impl Responder {
    shared::database::debug_panic();
    HttpResponse::InternalServerError().body("Debug panic triggered")
}
#[cfg(not(debug_assertions))]
async fn debug_panic() -> impl Responder {
    HttpResponse::Forbidden().body("Only available in debug environments")
}
