pub mod traits;
pub mod mock;
pub mod enigo_platform;
pub mod headless;
pub mod factory;

pub use traits::*;
pub use mock::MockPlatform;
pub use enigo_platform::EnigoPlatform;
pub use headless::HeadlessPlatform;
pub use factory::{PlatformFactory, PlatformCapabilities};