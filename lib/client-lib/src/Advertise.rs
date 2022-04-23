use bytes::{BufMut, BytesMut};
use custom_debug::Debug;
use getset::{CopyGetters, Getters, MutGetters, Setters};
use std::mem;
use crate::{
    Errors::ExoError,
    ClientLib::MqttSnClient,
};

#[derive(Debug, Clone, Getters, Setters, MutGetters, CopyGetters, Default)]
#[getset(get, set)]
pub struct Advertise {
    pub len: u8,
    #[debug(format = "0x{:x}")]
    pub msg_type: u8,
    pub gw_id: u8,
    pub duration: u16,
}

impl Advertise {
    fn constraint_len(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_msg_type(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_gw_id(_val: &u8) -> bool {
        //dbg!(_val);
        true
    }
    fn constraint_duration(_val: &u16) -> bool {
        //dbg!(_val);
        true
    }

    pub fn tx(gw_id: u8, duration: u16, client: &MqttSnClient) {
        
        let advertise = Advertise {
            len: 5, 
            msg_type: 0x00, 
            gw_id: 1,
            duration,            
        };
        let mut bytes_buf = BytesMut::with_capacity(advertise.len as usize);
        dbg!(advertise.clone());
        dbg!((bytes_buf.clone(), &advertise));
        advertise.try_write(&mut bytes_buf);
        dbg!(bytes_buf.clone());
        
        // transmit to network
        client
            .transmit_tx
            .send((client.remote_addr, bytes_buf.to_owned()));

    } 
        

    #[inline(always)]
    pub fn rx(
        buf: &[u8],
        size: usize,
        client: &MqttSnClient,
    ) -> Result<(), ExoError> {

        let (advertise, read_len) = Advertise::try_read(&buf, size).unwrap();
        dbg!(advertise.clone());                                                                   
        if read_len == size {
            // GwInfo::tx(advertise.gw_id,client);
            Ok(())
        } 
        else {
            return Err(ExoError::LenError(read_len, size));
        }
    }
}    