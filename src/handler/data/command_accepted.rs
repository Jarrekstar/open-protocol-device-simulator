use crate::protocol::field::FieldBuilder;
use crate::protocol::response_data::ResponseData;

pub struct CommandAccepted {
    pub accepted_mid: u32,
}

impl ResponseData for CommandAccepted {
    fn serialize(&self) -> Vec<u8> {
        let builder = FieldBuilder::new().add_int(None, self.accepted_mid as i32, 4);
        builder.build()
    }
}

impl CommandAccepted {
    pub fn new() -> Self {
        Self { accepted_mid: 0 }
    }
    
    pub fn with_mid(mid: u32) -> Self {
        Self { accepted_mid: mid }
    }
    
}

impl Default for CommandAccepted {
    fn default() -> Self {
        Self::new()
    }
}

