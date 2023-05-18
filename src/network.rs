use log::{info, log};
use sha3::{Digest, Sha3_256};
use blake3::Hasher;
use simplelog::{CombinedLogger, Config, Level, LevelFilter, TermLogger};
use std::collections::HashMap;
use std::convert::AsMut;
use std::io;

use kademlia_dht::{Key, Node};

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


const NUMBER_OF_NODES: usize = 33;

type NodeMap = Arc<Mutex<HashMap<usize, Node>>>;

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
    let mut node_map = NodeMap::new();
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
    let node_map = node_map.lock().unwrap();
    node_map[&index].clone()
}

pub fn get_node_id(node_map: &NodeMap, index: usize) -> usize {
    let node_map = node_map.lock().unwrap();
    node_map[&index].node_data().id
}

pub async fn insert_value(
    node_map: &NodeMap,
    index: usize,
    key: Key,
    value: String,
) {
    let mut node_map = node_map.lock().unwrap();
    node_map.get_mut(&index).unwrap().insert(key, value);
}

pub async fn get_value(
    node_map: &NodeMap,
    index: usize,
    key: Key,
) -> Option<String> {
    let node_map = node_map.lock().unwrap();
    node_map.get(&index).unwrap().get(&key)
}