use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

// States of the MQTT-SN machine,
// there are more as features require.
#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Debug, EnumIter, Copy, Clone)]
pub enum StateEnum {
    ACTIVE,
    DISCONNECTED,
    ASLEEP,
    AWAKE,
    LOST,
}
