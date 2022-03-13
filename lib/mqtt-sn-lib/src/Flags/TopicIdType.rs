#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TopicIdType {
   Normal = 0b00,
   PreDefined = 0b01,
   Short = 0b10,
   Reserved = 0b11,
}
 
impl TopicIdType {
#[inline(always)]
   pub fn new(flags: u8) -> TopicIdType {
       match flags & 0b0011 {
           0b00 => TopicIdType::Normal,
           0b01 => TopicIdType::PreDefined,
           0b10 => TopicIdType::Short,
           _ => TopicIdType::Reserved,
       }
   }
}