use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use storage::TransactionData;
use std::collections::HashMap;

use log::{info, log};
use sha3::{Digest, Sha3_256};
use blake3::{Hasher, Hash};
use simplelog::{CombinedLogger, Config, Level, LevelFilter, TermLogger};
use std::convert::AsMut;
use std::io;
use std::sync::Mutex;


mod key;
mod node;
mod protocol;
mod routing;
mod storage;

pub use self::key::Key;
pub use self::node::node_data::NodeData;
pub use self::node::Node;

/// The number of bytes in a key.
const KEY_LENGTH: usize = 32;

/// The maximum length of the message in bytes.
const MESSAGE_LENGTH: usize = 8196;

/// The maximum number of k-buckets in the routing table.
const ROUTING_TABLE_SIZE: usize = KEY_LENGTH * 8;

/// The maximum number of entries in a k-bucket.
const REPLICATION_PARAM: usize = 20;

/// The maximum number of active RPCs during `lookup_nodes`.
const CONCURRENCY_PARAM: usize = 3;

/// Request timeout time in milliseconds
const REQUEST_TIMEOUT: u64 = 5000;

/// Key-value pair expiration time in seconds
const KEY_EXPIRATION: u64 = 3600;

/// Bucket refresh interval in seconds
const BUCKET_REFRESH_INTERVAL: u64 = 3600;

  // let logger_config = Config {
   //     time: Some(Level::Error),
   //     level: Some(Level::Error),
   //     target: None,
   //     location: None,
   //     time_format: None,
   // };
   // CombinedLogger::init(vec![
   //     TermLogger::new(LevelFilter::Info, logger_config).unwrap()
   // ])
   // .unwrap();


const NUMBER_OF_NODES: usize = 9;

type NodeMap = HashMap<usize, Node>;

fn clone_into_array<A, T>(slice: &[T]) -> A
where
    A: Sized + Default + AsMut<[T]>,
    T: Clone,
{
    let mut a = Default::default();
    <A as AsMut<[T]>>::as_mut(&mut a).clone_from_slice(slice);
    a
}

fn get_key_sha(key: &str) -> Key {
    let mut hasher = Sha3_256::default();
    hasher.input(key.as_bytes());
    Key(clone_into_array(hasher.result().as_slice()))
}

fn get_key(key: &str) -> Key {
       let hash = blake3::hash(key.as_bytes());
       Key(clone_into_array(hash.as_bytes()))
    }

pub fn create_network(number_of_nodes: usize) -> NodeMap {
    let mut node_map = HashMap::new();
    let mut id = 0;
    for i in 0..number_of_nodes {
        if i == 0 {
            let n = Node::new(&"localhost".to_string(), &(8900 + i).to_string(), None);
            node_map.insert(id, n.clone());
        } else {
            let n = Node::new(
                &"localhost".to_string(),
                &(8900 + i).to_string(),
                Some(node_map[&0].node_data()),
            );
            node_map.insert(id, n.clone());
        }
        id += 1;
    }

    node_map
}
 
// node_map.get_mut(&index).unwrap().get(&key));

//  node_map.get_mut(&index).unwrap().insert(key, value);

pub fn get_node(node_map: &NodeMap, index: usize) -> Node {
    node_map[&index].clone()
}


pub async fn insert_value(
    node: &mut Node,
    key: String,
    value: TransactionData,
) {
    let key = get_key(&key);
    node.insert(key, &value);
}

pub async fn get_value(
    node: &mut Node,
    key: String,
) -> Option<TransactionData> {
    let key = get_key(&key);
    node.get(&key)
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

type Transactions = Mutex<Node>;

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

    let mut transactions: std::sync::MutexGuard<Node> = transactions.lock().unwrap();
    insert_value(&mut transactions, transaction_data.send_block.clone(), transaction_data.into_inner()).await;
    custom_http_response(CustomHttpResponse::TransactionProcessed)
}

async fn read_transaction(
    read_request: web::Json<ReadRequest>,
    transactions2: web::Data<Transactions>,
) -> impl Responder {
    if !validate_signature(&read_request) {
        return custom_http_response(CustomHttpResponse::InvalidSignature)
    }

    let mut transactions: std::sync::MutexGuard<Node> = transactions2.lock().unwrap();
    let maybe_value = get_value(&mut transactions, read_request.send_block.clone()).await;
    match maybe_value {
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

    let node_map = create_network(NUMBER_OF_NODES);
    let node = get_node(&node_map, 1);
    println!("{}", format!("{:?}", node.node_data()));
    let transactions = web::Data::new(Mutex::new(node));
    let node2 = get_node(&node_map, 5);
    println!("{}", format!("{:?}", node2.node_data()));
    let transactions2 = web::Data::new(Mutex::new(node2));

    println!("Starting server at http://localhost:8080");


    HttpServer::new( move || {
        
        App::new()
            .app_data(transactions.clone())
            .app_data(transactions2.clone())
            .service(web::resource("/send").route(web::post().to(process_transaction)))
            .service(web::resource("/receive").route(web::post().to(read_transaction)))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}