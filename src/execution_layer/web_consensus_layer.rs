use std::{ collections::HashMap, thread::sleep, time::Duration };

use lazy_static::lazy_static;
use handler::handle_request;
use tokio::sync::RwLock;
use transport_layer::{ msg::{NodeInfo, PluckingBody, MSG}, send, send_msg, INITFLAG, LOCALINFO, MSGPOOL };

pub mod transport_layer;
pub mod handler;
pub mod requester;

lazy_static! {
    pub static ref MAP: RwLock<Vec<(NodeInfo, Vec<usize>)>> = RwLock::new(Vec::new());
    pub static ref CONNECTED: RwLock<Vec<NodeInfo>> = RwLock::new(Vec::new());
    pub static ref WEBWIDTH: RwLock<usize> = RwLock::new(0);
    pub static ref SENDEDPOOL: RwLock<Vec<Vec<u8>>> = RwLock::new(Vec::new());
}

const MAX_SENDED_BUFFER_SIZE: usize = 10;

pub async fn start_web_consensus_layer(web_width: usize) {
    // check transportation layer
    let mut transport_layer_init_flag = INITFLAG.lock().await.clone();
    while !transport_layer_init_flag {
        sleep(Duration::from_secs(1));
        transport_layer_init_flag = INITFLAG.lock().await.clone();
    }
    //init web width
    *WEBWIDTH.write().await = web_width;
    //init map and connected
    let mut map = MAP.write().await;
    let mut connected = CONNECTED.write().await;
    let local_info = LOCALINFO.read().await;
    map.push((local_info.clone(), Vec::new()));
    for _ in 0..web_width {
        map[0].1.push(0);
        connected.push(local_info.clone());
    }
    drop(local_info);
    drop(map);
    drop(connected);
    //start listen and handle
    tokio::spawn(async {
        listen_and_handle().await;
    });
}

pub async fn listen_and_handle() {
    println!("start listen and handle requests...");
    loop {
        sleep(Duration::from_millis(300));
        let current_pool = MSGPOOL.read().await.clone();
        if current_pool.len() != 0 {
            let mut pool = MSGPOOL.write().await;
            for msg in current_pool {
                //TODO:try modify to multitasking,well,or not
                handle_request(msg).await;
            }
            pool.clear();
            drop(pool);
        }
    }
}

pub async fn resonating(
    connected: &Vec<NodeInfo>,
    sended_pool: &mut Vec<Vec<u8>>,
    max_sended_bufer: usize,
    msg: MSG
) {
    let msg_stamp: Vec<u8> = msg.hash_self();
    if sended_pool.contains(&msg_stamp) {
        return;
    }
    for node in connected {
        send_msg(msg.clone(), &node.listen_addr).await;
    }
    if sended_pool.len() >= max_sended_bufer {
        sended_pool.remove(0);
    }
    sended_pool.push(msg_stamp);
}

pub async fn plucking(msg:&MSG) ->bool{
    let mut body:PluckingBody=bincode::deserialize(&msg.data.body).expect("plucking body format error");
    if !(MAP.read().await[body.path[0]].0==LOCALINFO.read().await.to_owned()){
        println!("plucking path error:not me");
        return false;
    }
    body.path.remove(0);
    if body.path.len()==0{
        return true;
    }
    let target_node=MAP.read().await[body.path[0]].0.clone();
    if !CONNECTED.read().await.contains(&target_node){
        println!("plucking path error:next not reachable");
        return false;
    }
    send(msg.data.header.clone(), body, &target_node.listen_addr).await;
    false
}

pub async fn form_plucking_path() -> Vec<usize> {
    let mut start_node:usize=0;

    let map = MAP.read().await.to_owned();
    let mut hashed_map: HashMap<usize, Vec<usize>> = HashMap::new();
    let local_info=LOCALINFO.read().await.to_owned();
    for (i, node) in map.iter().enumerate() {
        if node.0==local_info{
            start_node=i;
        }
        hashed_map.insert(i, node.1.clone());
    }
    let path = find_eulerian_cycle(&mut hashed_map, start_node);

    path[0..find_index(&path, map.len() - 1)].to_vec()
}

fn find_eulerian_cycle(graph: &mut HashMap<usize, Vec<usize>>, start: usize) -> Vec<usize> {
    let mut stack = vec![start];
    let mut path = Vec::new();
    let mut current = start;

    while !stack.is_empty() {
        if let Some(neighbors) = graph.get_mut(&current) {
            if !neighbors.is_empty() {
                stack.push(current);
                current = neighbors.pop().unwrap();
            } else {
                path.push(current);
                current = stack.pop().unwrap();
            }
        } else {
            path.push(current);
            current = stack.pop().unwrap();
        }
    }

    path.reverse();
    path
}

fn find_index(path: &Vec<usize>, n: usize) -> usize {
    use std::collections::HashSet;

    let mut window = HashSet::new();
    let mut start = 0;

    for end in 0..path.len() {
        window.insert(path[end]);

        while window.len() == n + 1 {
            if path[end] == path[0] {
                return end+1;
            }
            window.remove(&path[start]);
            start += 1;
        }
    }
    path.len()
}