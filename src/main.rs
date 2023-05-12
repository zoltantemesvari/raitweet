use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex};

#[derive(Deserialize, Serialize)]
struct TransactionData {
    transaction: String,
    sender_account: String,
    payload: String,
    signature: String,
}

#[derive(Deserialize, Serialize)]
struct ReadRequest {
    transaction: String,
    sender_account: String,
    signature: String,
}

type Transactions = Mutex<HashMap<String, TransactionData>>;

async fn process_transaction(
    transaction_data: web::Json<TransactionData>,
    transactions: web::Data<Transactions>,
) -> impl Responder {
    if !validate_form(&transaction_data) {
        return HttpResponse::BadRequest().body("Invalid form or too large");
    }

    if !validate_signature(&transaction_data) {
        return HttpResponse::BadRequest().body("Invalid signature");
    }

    let mut transactions: std::sync::MutexGuard<HashMap<String, TransactionData>> = transactions.lock().unwrap();
    transactions.insert(transaction_data.transaction.clone(), transaction_data.into_inner());
    HttpResponse::Ok().body("Transaction processed")
}

async fn read_transaction(
    read_request: web::Json<ReadRequest>,
    transactions: web::Data<Transactions>,
) -> impl Responder {
    if !validate_signature(&read_request) {
        return HttpResponse::BadRequest().body("Invalid signature");
    }

    let transactions: std::sync::MutexGuard<HashMap<String, TransactionData>> = transactions.lock().unwrap();
    match transactions.get(&read_request.transaction) {
        Some(transaction_data) => {
            HttpResponse::Ok().json(transaction_data)
        }
        None => HttpResponse::NotFound().body("Transaction not found"),
    }
}

fn validate_form(transaction_data: &TransactionData) -> bool {
    let json_size = serde_json::to_string(transaction_data).unwrap().len();

    // Check overall size (max. 512 bytes)
    if json_size > 512 {
        return false;
    }

    // Add further form validation here if needed

    true
}

fn validate_signature<T: Serialize>(_data: &T) -> bool {
    // Replace this with the actual signature validation function
    // For this example, we'll just return true
    true
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    
    let transactions: web::Data<Mutex<HashMap<String, TransactionData>>> = web::Data::new(Transactions::new(HashMap::new()));

    println!("Starting server at http://localhost:8080");


    HttpServer::new( move || {
        
        App::new()
            .app_data(transactions.clone())
            .service(web::resource("/transaction").route(web::post().to(process_transaction)))
            .service(web::resource("/read").route(web::post().to(read_transaction)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}