pub mod traits;
pub mod mock;
pub mod enigo_platform;
pub mod headless;
pub mod factory;
pub mod cross_platform;
pub mod optimizations;
pub mod optimized_controller;

pub use traits::*;
pub use mock::MockPlatform;
pub use enigo_platform::EnigoPlatform;
pub use headless::HeadlessPlatform;
pub use factory::{PlatformFactory, PlatformCapabilities};
pub use cross_platform::{CrossPlatformController, BatchOperation};
pub use optimizations::PlatformOptimizations;
pub use optimized_controller::{OptimizedPlatformController, QueuedOperation, BatchProcessorStatus};