use std::error::Error;

pub struct PatchError {
    /// The internal error
    pub internal_error: Box<dyn Error>,
    /// The message which will be displayed in the client
    pub friendly_message: String,
    /// Controls the color of the error in the GUI
    pub level: PatchErrorLevel,
}

pub enum PatchErrorLevel {
    /// Displayed as yellow in the GUI
    Low,
    /// Displayed as red in the GUI
    High,
}

pub trait ToPatchError {
    /// Converts to a PatchError, with level High by default
    fn to_patch_error(self, friendly_message: &str) -> PatchError;
    /// Converts to a PatchError, with the level specified by a parameter
    fn to_patch_error_level(self, friendly_message: &str, level: PatchErrorLevel) -> PatchError;
}

impl<T> ToPatchError for T
where
    T: Into<Box<dyn Error>>,
{
    fn to_patch_error(self, friendly_message: &str) -> PatchError {
        self.to_patch_error_level(friendly_message, PatchErrorLevel::High)
    }

    fn to_patch_error_level(self, friendly_message: &str, level: PatchErrorLevel) -> PatchError {
        PatchError {
            internal_error: self.into(),
            friendly_message: friendly_message.to_owned(),
            level,
        }
    }
}
