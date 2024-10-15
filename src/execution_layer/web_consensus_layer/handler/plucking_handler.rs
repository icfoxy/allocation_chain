use crate::execution_layer::web_consensus_layer::{plucking, transport_layer::msg::{MSGHeader, PluckingType, MSG}};

pub async fn handle_plucking(msg:MSG){
    match msg.data.header {
        MSGHeader::Plucking(PluckingType::Ping)=>{
            handle_plucking_ping(msg).await;
        }
        _=>{}
    }
}

pub async fn handle_plucking_ping(msg:MSG){
    plucking(&msg).await;
}