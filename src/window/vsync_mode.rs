/// All the possible vsync settings
///
/// # Meaning
///
/// * `Disabled` - No vsync
/// * `Enabled` - Vsync is enabled
/// * `Adaptive` - Let the system decide
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum VsyncMode {
    Disabled,
    Enabled,
    Adaptive,
}
