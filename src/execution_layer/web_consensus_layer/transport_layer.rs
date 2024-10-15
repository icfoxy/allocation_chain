use std::{ net::UdpSocket, sync::Arc };

use lazy_static::lazy_static;
use msg::{ MSGHeader, NodeInfo, MSG };
use rand::rngs::OsRng;
use rsa::{ pkcs1::ToRsaPublicKey, RsaPrivateKey };
use serde::Serialize;
use tokio::sync::{ Mutex, RwLock };

pub mod msg;

lazy_static! {
    pub static ref MSGPOOL: RwLock<Vec<MSG>> = RwLock::new(Vec::new());
    pub static ref RECEIVEDPOOL: RwLock<Vec<Vec<u8>>> = RwLock::new(Vec::new());
    pub static ref LISTEN: RwLock<Arc<UdpSocket>> = RwLock::new(
        Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap())
    );
    pub static ref SEND: RwLock<Arc<UdpSocket>> = RwLock::new(
        Arc::new(UdpSocket::bind("127.0.0.1:0").unwrap())
    );
    pub static ref LOCALINFO: RwLock<NodeInfo> = RwLock::new(NodeInfo {
        public_key: "".to_string(),
        listen_addr: "".to_string(),
        send_addr: "".to_string(),
    });
    pub static ref RSAPRIVATEKEY: RwLock<RsaPrivateKey> = RwLock::new({
        let mut rng = OsRng;
        let bits = 512;
        RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate a key")
    });
    pub static ref INITFLAG: Mutex<bool> = Mutex::new(false);
}

const BUFFER_SIZE: usize = 4096;
const MAXRECEIVEPOOLSIZE:usize=10;

pub async fn start_transport_layer(
    listen_addr: String,
    send_addr: String,
    rsa_private_key: RsaPrivateKey
) {
    //create and lock init flag
    let mut flag = INITFLAG.lock().await;

    //init listen sockets
    let mut old_listen_socket = LISTEN.write().await;
    *old_listen_socket = Arc::new(UdpSocket::bind(listen_addr.clone()).expect("Udp bind error"));
    drop(old_listen_socket);

    //init send socket
    let mut old_send_socket = SEND.write().await;
    *old_send_socket = Arc::new(UdpSocket::bind(send_addr.clone()).expect("Udp bind error"));
    drop(old_send_socket);

    //init private_key
    let mut old_private = RSAPRIVATEKEY.write().await;
    *old_private = rsa_private_key;
    drop(old_private);

    //init local_info
    let local_info = NodeInfo {
        public_key: RSAPRIVATEKEY.read().await.to_public_key().to_pkcs1_pem().unwrap(),
        listen_addr: listen_addr,
        send_addr: send_addr,
    };
    let mut old_info = LOCALINFO.write().await;
    *old_info = local_info;
    drop(old_info);

    //start  listening
    tokio::spawn(async {
        listen(LISTEN.read().await.clone()).await;
    });

    //release init flag
    *flag = true;
    drop(flag);
}

async fn listen(listen_socket: Arc<UdpSocket>) {
    //init buffer
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

    //start listening
    println!("starting listening on {:?}", listen_socket.local_addr().unwrap());
    loop {
        let (msg_size, from) = listen_socket.recv_from(&mut buffer).expect("listening error");
        let msg: MSG = bincode::deserialize(&buffer[..msg_size]).expect("msg format error");

        //verify msg
        if !msg.verify_self() {
            println!("msg has been maliciously altered and won't be accepted");
        } else {
            let mut msg_pool = MSGPOOL.write().await;
            let hashed_msg = msg.hash_self();
            let mut received_pool = RECEIVEDPOOL.write().await;
            if !received_pool.contains(&hashed_msg) {
                println!("received:{:?} from {:?}", msg.data.header, from.to_string());
                msg_pool.push(msg);
                received_pool.push(hashed_msg);
                if received_pool.len()>MAXRECEIVEPOOLSIZE{
                    received_pool.remove(0);
                }
            }
            drop(msg_pool);
            drop(received_pool);
        }

        //clear buffer
        buffer = [0; BUFFER_SIZE];
    }
}

pub async fn send<T>(header: MSGHeader, body: T, target_addr: &String) where T: Serialize {
    let local_info = LOCALINFO.read().await.clone();
    let rsa_private_key = RSAPRIVATEKEY.read().await;
    let msg = MSG::new(header, body, local_info, &rsa_private_key);
    let encoded_msg: Vec<u8> = bincode::serialize(&msg).expect("msg encode error");
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    buffer[..encoded_msg.len()].copy_from_slice(&encoded_msg);
    let send_socket = SEND.write().await;
    send_socket.send_to(&buffer, target_addr).expect("sending error");
    drop(send_socket);
}

pub async fn send_msg(msg: MSG, target_addr: &String) {
    let encoded_msg: Vec<u8> = bincode::serialize(&msg).expect("msg encode error");
    let mut buffer: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    buffer[..encoded_msg.len()].copy_from_slice(&encoded_msg);
    let send_socket = SEND.write().await.clone();
    send_socket.send_to(&buffer, target_addr).expect("sending error");
    drop(send_socket);
}
