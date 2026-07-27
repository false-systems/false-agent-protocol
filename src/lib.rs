pub mod context;
pub mod digest;
pub mod error;
pub mod framing;
pub mod plugin;
pub mod work;
pub mod worker;

mod envelope;
mod id;
pub mod semantic;

pub use envelope::*;
pub use id::*;
pub use semantic::*;

pub const WORK_PACKET_FIXTURE: &str = include_str!("../fixtures/work-packet.v1.json");
pub const EXECUTION_CONTEXT_FIXTURE: &str = include_str!("../fixtures/execution-context.v1.json");
pub const WORKER_RESULT_FIXTURE: &str = include_str!("../fixtures/worker-result.v1.json");
pub const INITIALIZE_FIXTURE: &str = include_str!("../fixtures/initialize.v1.json");
pub const UNBOUND_CONTEXT_FIXTURE: &str = include_str!("../fixtures/unbound-context.v1.json");
pub const LAUNCH_PLAN_FIXTURE: &str = include_str!("../fixtures/launch-plan.v1.json");
