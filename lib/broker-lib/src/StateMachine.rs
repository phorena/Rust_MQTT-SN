use crate::MTU;
use crate::{
    MsgTypeConst, MSG_TYPE_CONNACK, MSG_TYPE_CONNECT, MSG_TYPE_MAX,
    MSG_TYPE_PUBACK, MSG_TYPE_PUBLISH, MSG_TYPE_SUBACK, MSG_TYPE_SUBSCRIBE,
};

/// Use of const u8 instead of enum:
/// 1. Portability
/// 2. In Rust converting from enum to number is simple,
///    but from number to enum is complicated, requires match/if.
/// 3. u8 allows flexibility, but enum is safer.
/// 4. Performance

pub type StateTypeConst = u8;
pub const STATE_ACTIVE: StateTypeConst = 0;
pub const STATE_DISCONNECT: StateTypeConst = 1;
pub const STATE_CONNECT_SENT: StateTypeConst = 2;
pub const STATE_LOST: StateTypeConst = 3;
pub const STATE_SLEEP: StateTypeConst = 4;
pub const STATE_AWAKE: StateTypeConst = 5;

pub const STATE_MAX: usize = 6;

#[derive(Debug, Clone)]
pub struct StateMachine {
    states: [[Option<StateTypeConst>; MSG_TYPE_MAX]; STATE_MAX],
}

// TODO
// convert states types into enum
// use 'as usize' to convert.
// don't need to convert msg_type because it fits into u8
impl StateMachine {
    pub fn new() -> Self {
        let mut state_machine = StateMachine {
            states: [[None; MSG_TYPE_MAX]; STATE_MAX],
        };
        // Initialize the state machine.
        // Use insert() to ensure the input & output states
        // are StateEnum and msg_type is u8.
        // TODO insert more states
        state_machine.insert(
            STATE_DISCONNECT,
            MSG_TYPE_CONNECT,
            STATE_CONNECT_SENT,
        );

        state_machine.insert(
            STATE_CONNECT_SENT,
            MSG_TYPE_CONNACK,
            STATE_ACTIVE,
        );

        state_machine.insert(STATE_ACTIVE, MSG_TYPE_PUBLISH, STATE_ACTIVE);
        state_machine
    }

    fn insert(
        &mut self,
        cur_state: StateTypeConst,
        msg_type: u8,
        next_state: StateTypeConst,
    ) {
        self.states[cur_state as usize][msg_type as usize] = Some(next_state);
    }

    // transition (cur_state, input_message) -> next_state
    #[inline(always)]
    pub fn transition(
        &self,
        cur_state: StateTypeConst,
        msg_type: u8,
    ) -> Option<StateTypeConst> {
        self.states[cur_state as usize][msg_type as usize]
    }
}
