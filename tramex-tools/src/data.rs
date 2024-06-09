//! This module contains the data structures used to store the data of the application.
use crate::interface::{interface_file::parser_rrc::RRCInfos, layer::Layer};
use core::fmt::Debug;

#[derive(Debug)]
/// Data structure to store Trace of the application.
pub struct Data {
    /// Vector of Trace.
    pub events: Vec<Trace>,
    /// Current index of the vector.
    pub current_index: usize,
}

impl Data {
    /// return the current trace
    pub fn get_current_trace(&self) -> Option<&Trace> {
        return self.events.get(self.current_index);
    }

    /// return if the index is different from the current index
    pub fn is_different_index(&self, index: usize) -> bool {
        if index == 0 {
            return true;
        }
        self.current_index != index
    }
}

impl Default for Data {
    fn default() -> Self {
        let default_data_size = 2048;
        Self {
            events: Vec::with_capacity(default_data_size),
            current_index: 0,
        }
    }
}

#[derive(Debug, Clone)]
/// Data structure to store Trace of the application.
pub struct Trace {
    /// Message type.
    /// Timestamp of the message.
    pub timestamp: u64,

    /// Layer of the message.
    pub layer: Layer,

    /// Message type.
    pub additional_infos: AdditionalInfos,

    /// Hexadecimal representation of the message.
    pub hexa: Vec<u8>,

    /// Text representation of the message from the API
    pub text: Option<Vec<String>>,
}

/// Data structure to store custom messages (from the amarisoft API)
#[derive(Debug, Clone)]
pub enum AdditionalInfos {
    /// RRC message
    RRCInfos(RRCInfos),
}
