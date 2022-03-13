#[derive(Debug, Clone)]
pub struct QoS {
   pub level: u8,
}
 
impl QoS {
#[inline(always)] 

   pub fn new(flags: u8) -> u8 {
       let level = (flags & 0b0110_0000) >> 5;
       let qos = QoS {
           level
       };
       qos.level
   }
}
// fn main(){
//     let q = QoS::new(0b0010_0000);
//     println!("{:?}", q);
// }
