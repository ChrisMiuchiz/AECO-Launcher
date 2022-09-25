pub enum PatchMessage {
    Error(String),
    Downloading(String, f32),
    Connecting(String),
}

pub enum GUIMessage {}
