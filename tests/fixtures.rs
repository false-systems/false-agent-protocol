use false_agent_protocol::{
    context::ExecutionContextEnvelope, plugin::InitializeParams, worker::LaunchPlan,
    INITIALIZE_FIXTURE, LAUNCH_PLAN_FIXTURE, UNBOUND_CONTEXT_FIXTURE,
};

#[test]
fn runtime_fixtures_are_compatible() {
    let initialize: InitializeParams = serde_json::from_str(INITIALIZE_FIXTURE).unwrap();
    assert_eq!(initialize.requested_capabilities[0].as_str(), "work.v1");

    let context: ExecutionContextEnvelope = serde_json::from_str(UNBOUND_CONTEXT_FIXTURE).unwrap();
    assert_eq!(
        context.context_digest,
        false_agent_protocol::digest::sha256_without(&context, "context_digest").unwrap()
    );
    context.validate().unwrap();
    assert!(context.work.is_none());

    let plan: LaunchPlan = serde_json::from_str(LAUNCH_PLAN_FIXTURE).unwrap();
    assert_eq!(plan.program, "kisko");
}
