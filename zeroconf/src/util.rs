#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ByteOrder {
    Default,
    BigEndian,
    LittleEndian,
}

impl Default for ByteOrder {
    fn default() -> Self {
        Self::Default
    }
}
