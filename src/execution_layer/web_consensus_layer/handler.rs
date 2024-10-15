use super::transport_layer::msg::{ MSGHeader, MSG };
use net_work_handler::{ handle_join_net, handle_ping, handle_show_current_map };
use plucking_handler::handle_plucking;

pub mod net_work_handler;
pub mod plucking_handler;

pub async fn handle_request(msg: MSG) {
    match msg.data.header {
        MSGHeader::Ping => {
            handle_ping(msg).await;
        }
        MSGHeader::JoinNet(_) => {
            handle_join_net(msg).await;
        }
        MSGHeader::ShowCurrentMap => {
            handle_show_current_map().await;
        }
        MSGHeader::Plucking(_)=>{
            handle_plucking(msg).await;
        }
    }
}
