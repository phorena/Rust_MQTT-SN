// use crate::Flags::{QoS::QoS,TopicIdType::TopicIdType};
use mqtt_sn_lib::Flags::{
    QoS::QoS,DUP::DUP,Retain::Retain, TopicIdType::TopicIdType
};

#[test]
fn test_QoS() {
    assert_eq!(QoS::new(0b0110_0000),3);
    assert_eq!(QoS::new(0b0010_0000),1);
    assert_eq!(QoS::new(0b0100_0000),2);
    assert_eq!(QoS::new(0b0000_0000),0);
}
#[test]
fn test_dup(){
    assert_eq!(DUP::new(0b1000_0000),1);
    assert_eq!(DUP::new(0b0000_0000),0);
}

#[test]
fn test_retain(){
    assert_eq!(Retain::new(0b0001_0000),1);
    assert_eq!(Retain::new(0b0000_0000),0);
}

#[test]
fn test_topicid(){
    assert_eq!(TopicIdType::new(0b0110_0000),TopicIdType::Normal);
    assert_eq!(TopicIdType::new(0b0110_0010),TopicIdType::Short);
    assert_eq!(TopicIdType::new(0b0110_0001),TopicIdType::PreDefined);
    assert_eq!(TopicIdType::new(0b0110_0011),TopicIdType::Reserved);
    assert_eq!(TopicIdType::new(0b0110_0000),TopicIdType::Normal);
    assert_eq!(TopicIdType::new(0b0010_0001),TopicIdType::PreDefined);
    assert_eq!(TopicIdType::new(0b0010_0010),TopicIdType::Short);
}