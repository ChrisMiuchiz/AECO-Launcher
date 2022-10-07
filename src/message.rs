pub enum PatchMessage {
    Error(String),
    Downloading(String, f32),
    Info(String),
    PatchStatus(PatchStatus),
}

pub enum PatchStatus {
    Finished,
    Working,
    Error,
    Close,
}

pub enum GUIMessage {
    Retry,
    Play,
    Close,
}
