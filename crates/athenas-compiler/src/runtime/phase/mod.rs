pub mod artifact;
pub mod core;
pub mod phases;

pub use artifact::ArtifactStore;
pub use core::{Phase, PhaseContext, PhaseId, PhaseOutput, PhaseResult, PhaseStatus};
pub use phases::register_all_phases;
