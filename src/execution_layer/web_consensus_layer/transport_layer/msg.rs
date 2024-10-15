use serde::{ Deserialize, Serialize };
use ring::digest::{ digest, SHA256 };
use rsa::{
    pkcs1:: FromRsaPublicKey ,
    PaddingScheme,
    PublicKey,
    RsaPrivateKey,
    RsaPublicKey,
    pkcs1::ToRsaPublicKey,
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum MSGHeader {
    Ping,
    ShowCurrentMap,
    Plucking(PluckingType),
    JoinNet(JoinNetStep),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PluckingBody{
    pub path:Vec<usize>,
    pub msg:MSG,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PluckingType{
    Ping
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum JoinNetStep {
    Request,
    SendQuest,
    ProveOfComputePower,
    UpdateWeb,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct NodeInfo {
    pub public_key: String,
    pub listen_addr: String,
    pub send_addr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MSGData {
    pub header: MSGHeader,
    pub body: Vec<u8>,
    pub info: NodeInfo,
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone,PartialEq)]
pub struct MSG {
    pub data: MSGData,
    pub sign: Vec<u8>,
}

impl MSG {
    pub fn new<T>(
        header: MSGHeader,
        body: T,
        info: NodeInfo,
        rsa_private_key: &RsaPrivateKey
    ) -> Self
        where T: Serialize
    {
        let public_key = RsaPublicKey::from(rsa_private_key).to_pkcs1_pem().unwrap();
        let data = MSGData {
            header: header,
            body: bincode::serialize(&body).expect("msg body format error"),
            info: info,
            public_key: public_key.clone(),
        };
        let encoded_data = bincode::serialize(&data).unwrap();
        let digest = digest(&SHA256, &encoded_data);
        let padding = PaddingScheme::new_pkcs1v15_sign(Some(rsa::Hash::SHA2_256));
        let sign = rsa_private_key.sign(padding, digest.as_ref()).unwrap();
        MSG {
            data: data,
            sign: sign,
        }
    }
    pub fn verify_self(&self) -> bool {
        let padding = PaddingScheme::new_pkcs1v15_sign(Some(rsa::Hash::SHA2_256));
        let rsa_public_key = RsaPublicKey::from_pkcs1_pem(&self.data.public_key).unwrap();
        let encoded_data = bincode::serialize(&self.data).unwrap();
        let digest = digest(&SHA256, &encoded_data);
        let result = rsa_public_key.verify(padding, digest.as_ref(), &self.sign);
        match result {
            Ok(_) => { true }
            Err(_) => { false }
        }
    }
    pub fn hash_self(&self)->Vec<u8>{
        #[derive(Serialize)]
        struct Stamp{
            header:MSGHeader,
            body:Vec<u8>,
            from:String,
        }
        let msg_stamp=Stamp{
            header:self.data.header.clone(),
            body:self.data.body.clone(),
            from:self.data.info.send_addr.clone(),
        };
        let encoded_data=bincode::serialize(&msg_stamp).unwrap();
        let digest=digest(&SHA256, &encoded_data);
        digest.as_ref().to_vec()
    }
}
