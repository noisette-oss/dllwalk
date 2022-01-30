fn main() {
    windows::build!(
        Windows::Win32::Foundation::*,
        Windows::Win32::System::Environment::*,
        Windows::Win32::System::Registry::*,
        Windows::Win32::System::SystemInformation::*,
        Windows::Win32::System::Diagnostics::Debug::*,
    );
}
