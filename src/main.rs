use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex};

#[derive(Deserialize, Serialize)]
struct TransactionData {
    send_block: String,
    payload: String,
    payload_hash: String,
    signature: String,
}

#[derive(Deserialize, Serialize)]
struct ReadRequest {
    receive_block: String,
    send_block: String,
    signature: String,
}

#[derive(Deserialize, Serialize)]
struct ReadResponse {
    payload: String
}

#[derive(Deserialize, Serialize)]

enum CustomHttpResponse {
    InvalidForm,
    InvalidSignature,
    TransactionNotFound,
    TransactionProcessed,
    GeneralError
}

type Transactions = Mutex<HashMap<String, TransactionData>>;

fn custom_http_response(response: CustomHttpResponse) -> HttpResponse {
    match response {
        CustomHttpResponse::InvalidForm => HttpResponse::BadRequest().body("Invalid form or too large"),
        CustomHttpResponse::InvalidSignature => HttpResponse::Unauthorized().body("Invalid signature"),
        CustomHttpResponse::TransactionNotFound => HttpResponse::NotFound().body("Transaction not found"),
        CustomHttpResponse::TransactionProcessed => HttpResponse::Created().body("Transaction processed"),
        CustomHttpResponse::GeneralError => HttpResponse::InternalServerError().body("General error"),
    }
}

async fn process_transaction(
    transaction_data: web::Json<TransactionData>,
    transactions: web::Data<Transactions>,
) -> impl Responder {
    if !validate_form(&transaction_data) {
        return custom_http_response(CustomHttpResponse::InvalidForm)
    }

    if !validate_signature(&transaction_data) {
        return custom_http_response(CustomHttpResponse::InvalidSignature)
    }

    let mut transactions: std::sync::MutexGuard<HashMap<String, TransactionData>> = transactions.lock().unwrap();
    transactions.insert(transaction_data.send_block.clone(), transaction_data.into_inner());
    custom_http_response(CustomHttpResponse::TransactionProcessed)
}

async fn read_transaction(
    read_request: web::Json<ReadRequest>,
    transactions: web::Data<Transactions>,
) -> impl Responder {
    if !validate_signature(&read_request) {
        return custom_http_response(CustomHttpResponse::InvalidSignature)
    }

    let transactions: std::sync::MutexGuard<HashMap<String, TransactionData>> = transactions.lock().unwrap();
    match transactions.get(&read_request.send_block) {
        Some(transaction_data) => {
            let transaction_data = ReadResponse {
                payload: transaction_data.payload.clone(),
            };
            HttpResponse::Ok().json(transaction_data)
        }
        None => custom_http_response(CustomHttpResponse::TransactionNotFound)
    }
}

fn validate_form(transaction_data: &TransactionData) -> bool {
    let data_json = serde_json::to_string(transaction_data);
    match data_json {
        Ok(data) =>  if data.len() > 512 {
                                    return false;
                                    }
                  else {()}
        Err(_) => return false
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
            .service(web::resource("/send").route(web::post().to(process_transaction)))
            .service(web::resource("/receive").route(web::post().to(read_transaction)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}