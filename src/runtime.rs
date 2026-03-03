//! Deterministic runtime driving a [`crate::graph::ServiceGraph`] with recovery policies.
//!
//! The runtime is intentionally small and deterministic:
//!
//! - no allocation
//! - no background threads
//! - no I/O
//!
//! This makes it suitable for OS kernels and for host-side unit tests.

use crate::{
    error::ServiceError,
    graph::{GraphError, MAX_SERVICES, ServiceGraph},
    health::HealthLevel,
    policy::{RecoveryAction, RecoveryPolicy},
    service::Service,
    state::ServiceState,
};

/// Outcome of a single [`FabricRuntime::step`] call.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FabricStepOutcome {
    /// No state changes occurred.
    Idle,
    /// At least one service state changed.
    Progress,
    /// A fatal error occurred; the embedding kernel should stop or escalate.
    Fatal(ServiceError),
}

/// A small, deterministic runtime that drives a service graph according to recovery policies.
///
/// # Slice alignment
///
/// The runtime stores per-node state and policy in caller-provided slices. The following
/// invariants must hold:
///
/// - `policies.len() == graph.nodes().len()`
/// - `states.len() == graph.nodes().len()`
///
/// This design keeps the runtime allocation-free and enables kernels to place these arrays
/// in statically allocated memory if desired.
pub struct FabricRuntime<'a, S: Service> {
    graph: ServiceGraph<'a, S>,
    policies: &'a [RecoveryPolicy],
    states: &'a mut [ServiceState],
}

impl<'a, S: Service> FabricRuntime<'a, S> {
    /// Creates a new fabric runtime from a graph and associated slices.
    pub fn new(
        graph: ServiceGraph<'a, S>,
        policies: &'a [RecoveryPolicy],
        states: &'a mut [ServiceState],
    ) -> Result<Self, GraphError> {
        let n = graph.nodes().len();
        if n > MAX_SERVICES {
            return Err(GraphError::TooManyServices);
        }
        if policies.len() != n || states.len() != n {
            return Err(GraphError::LengthMismatch);
        }
        Ok(FabricRuntime {
            graph,
            policies,
            states,
        })
    }

    /// Returns an immutable view of the graph.
    #[inline]
    pub fn graph(&self) -> &ServiceGraph<'a, S> {
        &self.graph
    }

    /// Returns the current lifecycle states.
    #[inline]
    pub fn states(&self) -> &[ServiceState] {
        self.states
    }

    /// Bootstraps all services in topological order.
    ///
    /// The runtime will:
    ///
    /// - validate the graph (ids, dependencies, cycles)
    /// - call `init` and then `start` on each service in dependency order
    /// - apply per-service initialization recovery policy on failures
    pub fn boot(&mut self, ctx: &mut S::Context) -> Result<(), ServiceError> {
        self.graph
            .validate()
            .map_err(|_| ServiceError::Implementation("service graph validation failed"))?;

        let n = self.graph.nodes().len();
        let mut order_buf = [0usize; MAX_SERVICES];
        let order = self
            .graph
            .topo_order(&mut order_buf[..n])
            .map_err(|_| ServiceError::Implementation("topological sort failed"))?;

        for &idx in order {
            if self.states[idx] != ServiceState::Created {
                continue;
            }

            self.states[idx] = ServiceState::Initializing;

            let policy = self.policies[idx];
            match policy.action_on_init_failure {
                RecoveryAction::Retry { max_retries } => {
                    self.init_start_with_retries(idx, ctx, max_retries)?;
                }
                RecoveryAction::Restart => {
                    // "Restart" during init is treated as a stop + a single retry.
                    let _ = self.graph.nodes_mut()[idx].service_mut().stop(ctx);
                    self.init_start_with_retries(idx, ctx, 1)?;
                }
                RecoveryAction::MarkDegraded => {
                    // Try once; if it fails, mark degraded and continue.
                    if self.init_start_once(idx, ctx).is_err() {
                        self.states[idx] = ServiceState::Degraded;
                    }
                }
                RecoveryAction::Fail => {
                    self.init_start_once(idx, ctx)?;
                }
            }
        }

        Ok(())
    }

    /// Requests a graceful shutdown of all services in reverse topological order.
    ///
    /// Shutdown is best-effort: stop errors are ignored by default.
    pub fn shutdown(&mut self, ctx: &mut S::Context) -> Result<(), ServiceError> {
        let n = self.graph.nodes().len();
        let mut order_buf = [0usize; MAX_SERVICES];
        let order = self
            .graph
            .topo_order(&mut order_buf[..n])
            .map_err(|_| ServiceError::Implementation("topological sort failed"))?;

        for &idx in order.iter().rev() {
            if self.states[idx].is_terminal() || self.states[idx] == ServiceState::Stopping {
                continue;
            }
            self.states[idx] = ServiceState::Stopping;
            let _ = self.graph.nodes_mut()[idx].service_mut().stop(ctx);
            self.states[idx] = ServiceState::Stopped;
        }

        Ok(())
    }

    /// Performs a single maintenance step.
    ///
    /// The runtime:
    ///
    /// - evaluates health for `Running` and `Degraded` services
    /// - transitions `Running → Degraded` if health is degraded
    /// - transitions `Degraded → Running` if health becomes ok
    /// - applies runtime recovery policy if health reports `Failed`
    pub fn step(&mut self, ctx: &mut S::Context) -> FabricStepOutcome {
        let mut progress = false;

        for idx in 0..self.graph.nodes().len() {
            match self.states[idx] {
                ServiceState::Running | ServiceState::Degraded => {
                    let health = self.graph.nodes()[idx].service().health();
                    match health.level() {
                        HealthLevel::Ok => {
                            if self.states[idx] == ServiceState::Degraded {
                                self.states[idx] = ServiceState::Running;
                                progress = true;
                            }
                        }
                        HealthLevel::Degraded => {
                            if self.states[idx] == ServiceState::Running {
                                self.states[idx] = ServiceState::Degraded;
                                progress = true;
                            }
                        }
                        HealthLevel::Failed => {
                            progress = true;
                            let msg = health.message().unwrap_or("service reported failed health");
                            let err = ServiceError::Implementation(msg);
                            match self.handle_runtime_failure(idx, err, ctx) {
                                Ok(()) => {}
                                Err(e) => return FabricStepOutcome::Fatal(e),
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if progress {
            FabricStepOutcome::Progress
        } else {
            FabricStepOutcome::Idle
        }
    }

    fn init_start_once(&mut self, idx: usize, ctx: &mut S::Context) -> Result<(), ServiceError> {
        let node = &mut self.graph.nodes_mut()[idx];
        node.service_mut().init(ctx)?;
        node.service_mut().start(ctx)?;
        self.states[idx] = ServiceState::Running;
        Ok(())
    }

    fn init_start_with_retries(
        &mut self,
        idx: usize,
        ctx: &mut S::Context,
        max_retries: u8,
    ) -> Result<(), ServiceError> {
        let attempts = (max_retries as usize) + 1;
        let mut last = None;
        for _ in 0..attempts {
            match self.init_start_once(idx, ctx) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last = Some(e);
                    let _ = self.graph.nodes_mut()[idx].service_mut().stop(ctx);
                    self.states[idx] = ServiceState::Initializing;
                }
            }
        }
        self.states[idx] = ServiceState::Failed;
        Err(last.unwrap_or(ServiceError::Implementation("init/start retries exhausted")))
    }

    fn handle_runtime_failure(
        &mut self,
        idx: usize,
        err: ServiceError,
        ctx: &mut S::Context,
    ) -> Result<(), ServiceError> {
        let policy = self.policies[idx];
        match policy.action_on_runtime_failure {
            RecoveryAction::Retry { max_retries } => {
                // Retry by attempting to call `start` again (service-specific meaning).
                let attempts = (max_retries as usize) + 1;
                let node = &mut self.graph.nodes_mut()[idx];
                for _ in 0..attempts {
                    match node.service_mut().start(ctx) {
                        Ok(()) => {
                            self.states[idx] = ServiceState::Running;
                            return Ok(());
                        }
                        Err(_) => {
                            // keep trying
                        }
                    }
                }
                self.states[idx] = ServiceState::Failed;
                Err(err)
            }
            RecoveryAction::Restart => {
                // Best-effort stop, then mark Created so the embedding kernel can call `boot` again,
                // or you can implement your own higher-level restart loop.
                self.states[idx] = ServiceState::Stopping;
                let _ = self.graph.nodes_mut()[idx].service_mut().stop(ctx);
                self.states[idx] = ServiceState::Created;
                Ok(())
            }
            RecoveryAction::MarkDegraded => {
                self.states[idx] = ServiceState::Degraded;
                Ok(())
            }
            RecoveryAction::Fail => {
                self.states[idx] = ServiceState::Failed;
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::{
        graph::ServiceNode,
        health::ServiceHealth,
        id::ServiceId,
        policy::{RecoveryAction, RecoveryPolicy},
    };

    #[derive(Copy, Clone)]
    struct OkSvc(&'static str);

    impl Service for OkSvc {
        type Context = ();

        fn id(&self) -> ServiceId {
            ServiceId::new(self.0)
        }

        fn init(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
            Ok(())
        }

        fn start(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
            Ok(())
        }

        fn stop(&mut self, _ctx: &mut Self::Context) -> Result<(), ServiceError> {
            Ok(())
        }

        fn health(&self) -> ServiceHealth {
            ServiceHealth::ok()
        }
    }

    #[test]
    fn boot_and_shutdown_ok() {
        let a = OkSvc("a");
        let b = OkSvc("b");
        static DEPS_B: &[ServiceId] = &[ServiceId::new("a")];

        let mut nodes = [
            ServiceNode::new(ServiceId::new("a"), &[], a),
            ServiceNode::new(ServiceId::new("b"), DEPS_B, b),
        ];
        let graph = ServiceGraph::new(&mut nodes).unwrap();

        let policies = [
            RecoveryPolicy {
                action_on_init_failure: RecoveryAction::Fail,
                action_on_runtime_failure: RecoveryAction::Fail,
            },
            RecoveryPolicy {
                action_on_init_failure: RecoveryAction::Fail,
                action_on_runtime_failure: RecoveryAction::Fail,
            },
        ];
        let mut states = [ServiceState::Created, ServiceState::Created];
        let mut rt = FabricRuntime::new(graph, &policies, &mut states).unwrap();

        let mut ctx = ();
        rt.boot(&mut ctx).unwrap();
        assert_eq!(rt.states(), &[ServiceState::Running, ServiceState::Running]);

        rt.shutdown(&mut ctx).unwrap();
        assert_eq!(rt.states(), &[ServiceState::Stopped, ServiceState::Stopped]);
    }
}
