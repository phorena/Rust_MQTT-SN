use std::sync::atomic::{AtomicU8, Ordering};
use std::{net::SocketAddr, sync::Arc};

/// Use of const u8 instead of enum:
/// 1. Portability
/// 2. In Rust converting from enum to number is simple,
///    but from number to enum is complicated, requires match/if.
/// 3. u8 allows flexibility, but enum is safer.
/// 4. Performance

type StateTypeConst = u8;
const STATE_ACTIVE: StateTypeConst = 0;
const STATE_DISCONNECT: StateTypeConst = 1;
const STATE_CONNECT_SENT: StateTypeConst = 2;
// const STATE_LOST: StateTypeConst = 3;
// const STATE_SLEEP: StateTypeConst = 4;
// const STATE_AWAKE: StateTypeConst = 5;

const STATE_MAX: usize = 6;

type MsgTypeConst = u8;
// const MSG_TYPE_CONNECT: MsgTypeConst = 0x4;
// const MSG_TYPE_CONNACK: MsgTypeConst = 0x5;
// const MSG_TYPE_PUBLISH: MsgTypeConst = 0xC; // should be 0, most popular
                                            // const MSG_TYPE_SUBSCRIBE: MsgTypeConst = 0x12;
                                            // const MSG_TYPE_SUBACK: MsgTypeConst = 0x13;
                                            // const MSG_TYPE_PUBACK: MsgTypeConst = 0xD;

// TODO fill in the rest
// const MSG_TYPE_WILLMSGRESP: MsgTypeConst = 0x1D; // 29

// 0x1E-0xFD reserved
// const MSG_TYPE_ENCAP_MSG: MsgTypeConst = 0xFE;
// XXX not an optimal choice because, array of MsgTypeConst
// must include 256 entries.
// For the 2x2 array [0..6][0..255] states,
// instead of array  [0..6][0..29] states.

const MSG_TYPE_MAX: usize = 256;

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
#[allow(non_camel_case_types)]
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct client_struct {
    state: u8,
}
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ClientStruct {
    // for performance, use lockfree structure
    state: Arc<AtomicU8>,
    addr: SocketAddr,
}

impl ClientStruct {
    pub fn new(state: u8, addr: SocketAddr) -> Self {
        ClientStruct {
            state: Arc::new(AtomicU8::new(state)),
            addr,
            //  state_machine,
        }
    }
    #[inline(always)]
    pub fn state_transition(
        &self,
        state_machine: &StateMachine,
        msg_type: u8,
    ) -> Option<u8> {
        // get cur_state
        let cur_state = self.state.load(Ordering::SeqCst);
        // transition to new state
        // dbg_fn!((cur_state, msg_type));
        let result = state_machine.transition(cur_state, msg_type).unwrap();
        // Save new result, might fail because another thread changed
        // the value, but very unlikely.
        // TODO check return value
        let _result = self.state.compare_exchange(
            cur_state,
            result,
            Ordering::Acquire,
            Ordering::Relaxed,
        );
        Some(result)
    }
    #[inline(always)]
    pub fn get_state(&self) -> u8 {
        self.state.load(Ordering::SeqCst)
    }
}
