pub struct Serial {
    pub data: u8,
    pub transfer_control: u8,
}

impl Serial {
    pub fn new() -> Serial {
        Serial {
            data: 0,
            transfer_control: 0,
        }
    }
}
