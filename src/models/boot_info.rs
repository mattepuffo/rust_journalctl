#[derive(Debug, Clone)]
pub struct BootInfo {
    pub boot_id: String,
    pub boot_offset: i32,
    pub first_entry: String,
    pub last_entry: String,
}