
use rand::{ rngs::OsRng, seq::SliceRandom };

use crate::execution_layer::web_consensus_layer::{resonating, transport_layer::{
    
    msg::{ JoinNetStep, MSGHeader, NodeInfo, MSG },
    send,
    LOCALINFO,
    RSAPRIVATEKEY,
}, CONNECTED, MAP, MAX_SENDED_BUFFER_SIZE, SENDEDPOOL, WEBWIDTH};


pub async fn handle_ping(msg: MSG) {
    resonating(
        &*CONNECTED.read().await,
        &mut *SENDEDPOOL.write().await,
        MAX_SENDED_BUFFER_SIZE,
        msg
    ).await;
}

pub async fn handle_show_current_map() {
    let map = MAP.read().await.to_owned();
    let mut i: usize = 0;
    for node in map {
        println!("[{}] ({} {}) {:?}", i, node.0.listen_addr, node.0.send_addr, node.1);
        i += 1;
    }
}

pub async fn handle_join_net(msg: MSG) {
    match msg.data.header {
        MSGHeader::JoinNet(JoinNetStep::Request) => {
            //TODO:create quest
            send(MSGHeader::JoinNet(JoinNetStep::SendQuest), "", &msg.data.info.listen_addr).await;
        }
        MSGHeader::JoinNet(JoinNetStep::SendQuest) => {
            //TODO:prove compute power
            send(
                MSGHeader::JoinNet(JoinNetStep::ProveOfComputePower),
                ((), LOCALINFO.read().await.clone()),
                &msg.data.info.listen_addr
            ).await;
        }
        MSGHeader::JoinNet(JoinNetStep::ProveOfComputePower) => {
            //TODO:verify prove
            if !verify_compute_power_prove() {
                return;
            }

            //update web
            let (_, new_node): ((), NodeInfo) = bincode
                ::deserialize(&msg.data.body)
                .expect("new node info format error");
            add_new_node(new_node).await;

            //broadcast new web
            resonating(
                &*CONNECTED.read().await,
                &mut *SENDEDPOOL.write().await,
                MAX_SENDED_BUFFER_SIZE,
                MSG::new(
                    MSGHeader::JoinNet(JoinNetStep::UpdateWeb),
                    ((), MAP.read().await.clone()),
                    LOCALINFO.read().await.clone(),
                    &*RSAPRIVATEKEY.read().await
                )
            ).await;
        }
        MSGHeader::JoinNet(JoinNetStep::UpdateWeb) => {
            //verify web
            if !verify_web() {
                return;
            }
            let (_, new_map): ((), Vec<(NodeInfo, Vec<usize>)>) = bincode
                ::deserialize(&msg.data.body)
                .expect("new map format error");
            *MAP.write().await = new_map;
            update_connected().await;

            resonating(
                &*CONNECTED.read().await,
                &mut *SENDEDPOOL.write().await,
                MAX_SENDED_BUFFER_SIZE,
                msg
            ).await;
        }
        _ => {}
    };
}

//TODO:verify prove
fn verify_compute_power_prove() -> bool {
    true
}

//TODO:verify web
fn verify_web() -> bool {
    true
}

async fn update_connected() {
    let map = MAP.read().await.clone();

    let mut connected = CONNECTED.write().await;
    connected.clear();
    let local_info = LOCALINFO.read().await.clone();
    for nodes in map.clone() {
        if nodes.0 == local_info {
            for index in nodes.1 {
                connected.push(map[index].0.clone());
            }
        }
    }
}

fn random_vec(max: usize, num: usize) -> Vec<usize> {
    let mut rng = OsRng;
    let mut vec: Vec<usize> = (0..=max).collect();

    if max < num {
        vec.shuffle(&mut rng);
        vec
    } else {
        vec.shuffle(&mut rng);
        vec.into_iter().take(num).collect()
    }
}

pub async fn add_new_node(new_node: NodeInfo) {
    let web_width = WEBWIDTH.read().await.clone();
    let mut map = MAP.read().await.clone();
    let len = map.len();
    let mut need_return: Vec<usize> = Vec::new();
    for _ in 0..web_width {
        need_return.push(len);
    }
    let rand_nodes = random_vec(len - 1, web_width);
    for i in rand_nodes {
        if i == map.len() {
            break;
        }
        let mut selected: usize = 0;
        while selected < map[i].1.len() {
            if !need_return.contains(&map[i].1[selected]) {
                break;
            }
            selected += 1;
        }
        need_return.push(map[i].1[selected]);
        map[i].1.remove(selected);
        map[i].1.push(len);
    }

    need_return = need_return[need_return.len() - web_width..].to_vec();
    map.push((new_node, need_return));

    let mut connected = CONNECTED.write().await;
    connected.clear();
    let local_info = LOCALINFO.read().await.clone();
    for nodes in map.clone() {
        if nodes.0 == local_info {
            for index in nodes.1 {
                connected.push(map[index].0.clone());
            }
        }
    }

    *MAP.write().await = map;
}