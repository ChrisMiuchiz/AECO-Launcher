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
    GameLaunched,
}

pub enum GUIMessage {
    Retry,
    Play,
}
