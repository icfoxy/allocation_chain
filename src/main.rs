use std::{env, thread::sleep, time::Duration};

use execution_layer::web_consensus_layer::{
     requester::{plucking_ping, request_join_net}, start_web_consensus_layer, transport_layer::start_transport_layer
};
use rsa::RsaPrivateKey;
use rand::rngs::OsRng;
use color_eyre::eyre::Result;
mod execution_layer;

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut rng = OsRng;
    let bits = 512;
    let private_key = RsaPrivateKey::new(&mut rng, bits).expect("Failed to generate a key");

    start_transport_layer(args[1].clone(), args[2].clone(), private_key).await;
    start_web_consensus_layer(3).await;

    if args.len() > 3 {
        request_join_net(args[3].clone()).await;
    }

    sleep(Duration::from_secs(5));
    if args.len() >4{
        plucking_ping().await;
    }

    loop {
        
    }

    // let node1=NodeInfo{
    //     public_key:"".to_string(),
    //     listen_addr:"".to_string(),
    //     send_addr:"".to_string(),
    // };

    // let node2=NodeInfo{
    //     public_key:"".to_string(),
    //     listen_addr:"".to_string(),
    //     send_addr:"".to_string(),
    // };

    // let node3=NodeInfo{
    //     public_key:"".to_string(),
    //     listen_addr:"".to_string(),
    //     send_addr:"".to_string(),
    // };

    // let node4=NodeInfo{
    //     public_key:"".to_string(),
    //     listen_addr:"".to_string(),
    //     send_addr:"".to_string(),
    // };

    // let node5=NodeInfo{
    //     public_key:"".to_string(),
    //     listen_addr:"".to_string(),
    //     send_addr:"".to_string(),
    // };


    // start_transport_layer("127.0.0.1:8089".to_string(), "127.0.0.1:8090".to_string(), private_key).await;
    // start_web_consensus_layer(3).await;

    // add_new_node(node1).await;
    // add_new_node(node2).await;
    // add_new_node(node3).await;
    // add_new_node(node4).await;
    // add_new_node(node5).await;

    // let mut i=0;
    // for node in MAP.read().await.clone(){
    //     println!("[{}] : {:?}",i,node.1);
    //     i+=1;
    // };

    // println!("{:?}",form_plucking_path(0).await);
    // Ok(())
}
