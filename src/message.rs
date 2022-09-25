pub enum PatchMessage {
    Error(String),
    Downloading(String, f32),
}

pub enum GUIMessage {}
