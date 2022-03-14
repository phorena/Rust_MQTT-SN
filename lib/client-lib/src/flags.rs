// flags
//
pub type DupConst = u8;
pub const DUP_FALSE: DupConst = 0b_0_00_0_0_0_00;
pub const DUP_TRUE: DupConst = 0b_1_00_0_0_0_00;

pub type QoSConst = u8;
pub const QOS_LEVEL_0: QoSConst = 0b_0_00_0_0_0_00;
pub const QOS_LEVEL_1: QoSConst = 0b_0_01_0_0_0_00;
pub const QOS_LEVEL_2: QoSConst = 0b_0_10_0_0_0_00;
pub const QOS_LEVEL_3: QoSConst = 0b_0_11_0_0_0_00;

pub type RetainConst = u8;
pub const RETAIN_FALSE: RetainConst = 0b_0_00_0_0_0_00;
pub const RETAIN_TRUE: RetainConst = 0b_0_00_1_0_0_00;

pub type WillConst = u8;
pub const WILL_FALSE: WillConst = 0b_0_00_0_0_0_00;
pub const WILL_TRUE: WillConst = 0b_0_00_0_1_0_00;

pub type CleanSessionConst = u8;
pub const CLEAN_SESSION_FALSE: CleanSessionConst = 0b_0_00_0_0_0_00;
pub const CLEAN_SESSION_TRUE: CleanSessionConst = 0b_0_00_0_0_1_00;

pub type TopicIdTypeConst = u8;
pub const TOPIC_ID_TYPE_NORNAL: TopicIdTypeConst = 0b_0_00_0_0_0_00;
pub const TOPIC_ID_TYPE_PRE_DEFINED: TopicIdTypeConst = 0b_0_00_0_0_0_01;
pub const TOPIC_ID_TYPE_SHORT: TopicIdTypeConst = 0b_0_00_0_0_0_10;
pub const TOPIC_ID_TYPE_RESERVED: TopicIdTypeConst = 0b_0_00_0_0_0_11;

#[inline(always)]
pub fn flag_is_dup(input: u8) -> bool {
    (input & 0b1_0000000) != 0
}
#[inline(always)]
pub fn flag_qos_level(input: u8) -> QoSConst {
    input & 0b0_11_00000
}
#[inline(always)]
pub fn flag_is_retain(input: u8) -> bool {
    (input & 0b000_1_0000) != 0
}
#[inline(always)]
pub fn flag_is_will(input: u8) -> bool {
    (input & 0b0000_1_000) != 0
}
#[inline(always)]
pub fn flag_is_clean_session(input: u8) -> bool {
    (input & 0b00000_1_00) != 0
}
#[inline(always)]
pub fn flag_sess_id_type(input: u8) -> TopicIdTypeConst {
    input & 0b11
}
#[inline(always)]
pub fn flags_set(
    dup: DupConst,
    qos: QoSConst,
    retain: RetainConst,
    will: WillConst,
    clean_session: CleanSessionConst,
    topic_id_type: TopicIdTypeConst,
) -> u8 {
    dup | qos | retain | will | clean_session | topic_id_type
}
#[inline(always)]
pub fn flag_set_dup(bytes: &[u8], dup: DupConst) -> u8 {
    dup | bytes[2]
}
