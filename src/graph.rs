//! Static, allocation-free service dependency graphs.
//!
//! A `ServiceGraph` is a *static* description of services and dependencies.
//! It is designed for OS development where:
//!
//! - allocation is undesirable or unavailable early in boot
//! - determinism is important for testing and CI
//! - the set of services is typically known at compile time
//!
//! The graph is expected to be a DAG (directed acyclic graph).
//! Dependencies are expressed as:
//!
//! > "This service depends on these other services being started first."
//!
//! Concretely: if `B` depends on `A`, then the graph contains an edge `A → B`
//! and `A` must appear *before* `B` in topological order.

use crate::{id::ServiceId, service::Service};

/// Maximum number of services supported by the reference implementation.
///
/// This crate is intentionally allocation-free; to keep the implementation
/// small and deterministic, it uses fixed-size internal arrays sized by this
/// constant.
///
/// If you need more services, you can increase this constant and rebuild.
pub const MAX_SERVICES: usize = 64;

/// Describes a single node in the service graph.
///
/// The node does not allocate and does not own any dynamic collections.
/// Dependencies are expressed as a static slice of [`ServiceId`].
///
/// # Why store `id` separately from `service.id()`?
///
/// Many kernels prefer declaring service IDs as constants and wiring graphs
/// before services are fully constructed. By keeping an explicit `id` here,
/// you can describe the graph independent of the service's internal structure.
///
/// The graph validator also checks that `id == service.id()` to catch wiring
/// mistakes early.
pub struct ServiceNode<S: Service> {
    id: ServiceId,
    deps: &'static [ServiceId],
    service: S,
}

impl<S: Service> ServiceNode<S> {
    /// Creates a new service node with a static dependency list.
    pub const fn new(id: ServiceId, deps: &'static [ServiceId], service: S) -> Self {
        ServiceNode { id, deps, service }
    }

    /// Returns this node's declared identifier.
    #[inline]
    pub const fn id(&self) -> ServiceId {
        self.id
    }

    /// Returns the static slice of dependencies for this node.
    #[inline]
    pub const fn deps(&self) -> &'static [ServiceId] {
        self.deps
    }

    /// Returns an immutable reference to the underlying service.
    #[inline]
    pub fn service(&self) -> &S {
        &self.service
    }

    /// Returns a mutable reference to the underlying service.
    #[inline]
    pub fn service_mut(&mut self) -> &mut S {
        &mut self.service
    }
}

/// Errors that can occur while constructing or validating a service graph.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GraphError {
    /// A node referenced a dependency that does not exist in the graph.
    MissingDependency(&'static str),
    /// The graph contains at least one dependency cycle.
    CyclicDependency,
    /// A caller-provided buffer was too small.
    BufferTooSmall,
    /// The graph exceeds the supported `MAX_SERVICES` limit.
    TooManyServices,
    /// The lengths of associated slices did not match the number of nodes.
    LengthMismatch,
    /// A node's declared id did not match the service-provided id.
    IdMismatch(&'static str),
}

/// A static, allocation-free description of a service dependency graph.
///
/// The graph is expected to be a DAG; validation will fail otherwise.
pub struct ServiceGraph<'a, S: Service> {
    nodes: &'a mut [ServiceNode<S>],
}

impl<'a, S: Service> ServiceGraph<'a, S> {
    /// Creates a new graph from the provided slice of nodes.
    ///
    /// This function does not perform validation; use [`ServiceGraph::validate`] to check for:
    ///
    /// - missing dependencies
    /// - cycles
    /// - id mismatches (`node.id` vs `service.id()`)
    pub fn new(nodes: &'a mut [ServiceNode<S>]) -> Result<Self, GraphError> {
        if nodes.len() > MAX_SERVICES {
            return Err(GraphError::TooManyServices);
        }
        Ok(ServiceGraph { nodes })
    }

    /// Returns the underlying nodes.
    #[inline]
    pub fn nodes(&self) -> &[ServiceNode<S>] {
        self.nodes
    }

    /// Returns a mutable reference to the underlying nodes.
    #[inline]
    pub fn nodes_mut(&mut self) -> &mut [ServiceNode<S>] {
        self.nodes
    }

    /// Validates the graph.
    ///
    /// Validation ensures:
    ///
    /// - all declared dependencies are present in the graph
    /// - the graph contains no cycles
    /// - each node's declared id matches its service's id
    pub fn validate(&self) -> Result<(), GraphError> {
        self.check_id_matches()?;
        self.check_missing_deps()?;
        self.check_cycles()
    }

    /// Computes a topological ordering of node indices into `buffer`.
    ///
    /// The returned slice contains indices into `self.nodes()` such that
    /// for every dependency edge `A → B`, `A` appears before `B`.
    ///
    /// The buffer must be at least as large as the number of nodes.
    ///
    /// # Determinism
    ///
    /// When multiple nodes have indegree 0, the implementation picks them in
    /// ascending index order, making the output deterministic.
    pub fn topo_order<'b>(&self, buffer: &'b mut [usize]) -> Result<&'b [usize], GraphError> {
        let n = self.nodes.len();
        if n > MAX_SERVICES {
            return Err(GraphError::TooManyServices);
        }
        if buffer.len() < n {
            return Err(GraphError::BufferTooSmall);
        }

        // indegree[i] = number of dependencies of node i
        let mut indegree = [0u8; MAX_SERVICES];
        for (i, node) in self.nodes.iter().enumerate() {
            let d = node.deps().len();
            if d > u8::MAX as usize {
                return Err(GraphError::LengthMismatch);
            }
            indegree[i] = d as u8;
        }

        // Simple queue for nodes with indegree 0.
        let mut q = [0usize; MAX_SERVICES];
        let mut qh = 0usize;
        let mut qt = 0usize;

        for (i, deg) in indegree.iter().enumerate().take(n) {
            if *deg == 0 {
                q[qt] = i;
                qt += 1;
            }
        }

        let mut out_len = 0usize;
        while qh < qt {
            let u = q[qh];
            qh += 1;

            buffer[out_len] = u;
            out_len += 1;

            // For every node v that depends on u, decrement indegree[v].
            let uid = self.nodes[u].id();
            for (v, node) in self.nodes.iter().enumerate() {
                if node.deps().contains(&uid) {
                    // safe: indegree[v] starts at deps len and is decremented once per dep
                    indegree[v] = indegree[v].saturating_sub(1);
                    if indegree[v] == 0 {
                        q[qt] = v;
                        qt += 1;
                    }
                }
            }
        }

        if out_len != n {
            return Err(GraphError::CyclicDependency);
        }

        Ok(&buffer[..out_len])
    }

    fn check_id_matches(&self) -> Result<(), GraphError> {
        for node in self.nodes.iter() {
            let declared = node.id();
            let actual = node.service().id();
            if declared != actual {
                return Err(GraphError::IdMismatch(declared.as_str()));
            }
        }
        Ok(())
    }

    fn check_missing_deps(&self) -> Result<(), GraphError> {
        for node in self.nodes.iter() {
            for dep in node.deps() {
                if !self.nodes.iter().any(|n| n.id() == *dep) {
                    return Err(GraphError::MissingDependency(dep.as_str()));
                }
            }
        }
        Ok(())
    }

    fn check_cycles(&self) -> Result<(), GraphError> {
        let mut buf = [0usize; MAX_SERVICES];
        let n = self.nodes.len();
        if n == 0 {
            return Ok(());
        }
        self.topo_order(&mut buf[..n]).map(|_| ())
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use crate::{error::ServiceError, health::ServiceHealth};

    #[derive(Copy, Clone)]
    struct T(&'static str);

    impl Service for T {
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
    fn topo_order_dependencies_first() {
        static DEPS_B: &[ServiceId] = &[ServiceId::new("a")];

        let mut nodes = [
            ServiceNode::new(ServiceId::new("a"), &[], T("a")),
            ServiceNode::new(ServiceId::new("b"), DEPS_B, T("b")),
        ];

        let g = ServiceGraph::new(&mut nodes).unwrap();
        g.validate().unwrap();

        let mut out = [0usize; 2];
        let ord = g.topo_order(&mut out).unwrap();
        assert_eq!(ord, &[0, 1]);
    }

    #[test]
    fn detect_missing_dep() {
        static DEPS: &[ServiceId] = &[ServiceId::new("nope")];
        let mut nodes = [ServiceNode::new(ServiceId::new("a"), DEPS, T("a"))];
        let g = ServiceGraph::new(&mut nodes).unwrap();
        assert_eq!(g.validate(), Err(GraphError::MissingDependency("nope")));
    }

    #[test]
    fn detect_cycle() {
        static DEPS_A: &[ServiceId] = &[ServiceId::new("b")];
        static DEPS_B: &[ServiceId] = &[ServiceId::new("a")];
        let mut nodes = [
            ServiceNode::new(ServiceId::new("a"), DEPS_A, T("a")),
            ServiceNode::new(ServiceId::new("b"), DEPS_B, T("b")),
        ];
        let g = ServiceGraph::new(&mut nodes).unwrap();
        assert_eq!(g.validate(), Err(GraphError::CyclicDependency));
    }
}
