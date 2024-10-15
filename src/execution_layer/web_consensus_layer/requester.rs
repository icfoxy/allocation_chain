use super::{form_plucking_path, plucking, resonating, transport_layer::{msg::{JoinNetStep, MSGHeader, PluckingBody, PluckingType, MSG}, send, LOCALINFO, RSAPRIVATEKEY}, CONNECTED, SENDEDPOOL};

#[allow(dead_code)]
pub async fn ping(){
    resonating(
        &*CONNECTED.read().await,
        &mut *SENDEDPOOL.write().await,
        10,
        MSG::new(MSGHeader::Ping, "Ping", LOCALINFO.read().await.clone(), &*RSAPRIVATEKEY.read().await)
    ).await;
}

pub async fn plucking_ping(){
    let path=form_plucking_path().await;
    let msg=MSG::new(MSGHeader::Plucking(PluckingType::Ping), PluckingBody{
        path:path,
        msg:MSG::new(MSGHeader::Ping, "Ping", LOCALINFO.read().await.clone(), &*RSAPRIVATEKEY.read().await)
    }, LOCALINFO.read().await.clone(),&*RSAPRIVATEKEY.read().await);
    plucking(&msg).await;
}

#[allow(dead_code)]
pub async fn request_join_net(target_addr:String){
    send(MSGHeader::JoinNet(JoinNetStep::Request), "", &target_addr).await;
}