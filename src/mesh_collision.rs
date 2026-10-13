//! Static, particle-center collision boundaries in metres. Surface admission
//! does not infer a volume; solid admission is deliberately limited to a
//! connected, consistently wound, watertight convex shell. Response is a
//! conservative swept contact, not an SPH pressure/shear boundary or body solver.
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

pub const MAX_MESH_VERTICES: usize = 100_000;
pub const MAX_MESH_TRIANGLES: usize = 100_000;
const MAX_CONVEX_CHECKS: usize = 4_000_000;
const MAX_CONVEX_TRIANGLES: usize = 4096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshPolicy {
    /// Two-sided, zero-thickness barrier. Open and disconnected surfaces allowed.
    TwoSidedSurface,
    /// One connected, closed convex solid. Indexed edges must pair oppositely.
    ConvexSolid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MeshCollisionError {
    Empty,
    ResourceLimit,
    AllocationFailed,
    NonFiniteOrExtremeCoordinate,
    InvalidIndex,
    DegenerateTriangle,
    IllConditionedBounds,
    NotWatertightOrConsistentlyWound,
    DisconnectedOrUnusedVertex,
    NonManifoldVertexOrDuplicatePosition,
    NotConvex,
    NonFiniteQuery,
    QueryPrecisionUnsupported,
    QueryStartsInsideSolid,
}
impl fmt::Display for MeshCollisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Collision mesh: {self:?}")
    }
}
impl std::error::Error for MeshCollisionError {}

#[derive(Debug, Clone, Copy)]
struct Bounds {
    low: [f64; 3],
    high: [f64; 3],
}
impl Bounds {
    fn empty() -> Self {
        Self {
            low: [f64::INFINITY; 3],
            high: [f64::NEG_INFINITY; 3],
        }
    }
    fn include(&mut self, point: [f64; 3]) {
        for (d, value) in point.into_iter().enumerate() {
            self.low[d] = self.low[d].min(value);
            self.high[d] = self.high[d].max(value);
        }
    }
    fn intersects(&self, start: [f64; 3], end: [f64; 3], eps: f64) -> bool {
        let mut near: f64 = 0.0;
        let mut far: f64 = 1.0;
        for d in 0..3 {
            let delta = end[d] - start[d];
            if delta == 0.0 {
                if start[d] < self.low[d] - eps || start[d] > self.high[d] + eps {
                    return false;
                }
            } else {
                let a = (self.low[d] - eps - start[d]) / delta;
                let b = (self.high[d] + eps - start[d]) / delta;
                near = near.max(a.min(b));
                far = far.min(a.max(b));
                if near > far {
                    return false;
                }
            }
        }
        true
    }
}
#[derive(Debug)]
struct Node {
    bounds: Bounds,
    begin: usize,
    end: usize,
    children: Option<[usize; 2]>,
}

/// Immutable validated geometry and balanced BVH. Construction owns bounded
/// allocations; collision queries allocate nothing. Input vector capacities are
/// included in `allocated_bytes`, including caller over-allocation.
#[derive(Debug)]
pub struct CollisionMesh {
    vertices: Vec<[f64; 3]>,
    triangles: Vec<[usize; 3]>,
    normals: Vec<[f64; 3]>,
    order: Vec<usize>,
    nodes: Vec<Node>,
    bounds: Bounds,
    policy: MeshPolicy,
    epsilon: f64,
}
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|d| a[d] - b[d])
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|d| a[d] * b[d]).sum()
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn reserve<T>(n: usize) -> Result<Vec<T>, MeshCollisionError> {
    let mut out = Vec::new();
    out.try_reserve_exact(n)
        .map_err(|_| MeshCollisionError::AllocationFailed)?;
    Ok(out)
}
impl CollisionMesh {
    /// Rejects nonfinite/extreme coordinates (absolute value > 1e9 metres),
    /// tiny/ill-conditioned triangles, invalid topology for solids and excessive
    /// work. No automatic welding, hole filling or arbitrary-solid inference.
    pub fn new(
        vertices: Vec<[f64; 3]>,
        triangles: Vec<[usize; 3]>,
        policy: MeshPolicy,
    ) -> Result<Self, MeshCollisionError> {
        if vertices.is_empty() || triangles.is_empty() {
            return Err(MeshCollisionError::Empty);
        }
        if vertices.capacity() > MAX_MESH_VERTICES || triangles.capacity() > MAX_MESH_TRIANGLES {
            return Err(MeshCollisionError::ResourceLimit);
        }
        if policy == MeshPolicy::ConvexSolid
            && (triangles.len() > MAX_CONVEX_TRIANGLES
                || vertices.len().saturating_mul(triangles.len()) > MAX_CONVEX_CHECKS)
        {
            return Err(MeshCollisionError::ResourceLimit);
        }
        let mut bounds = Bounds::empty();
        for &p in &vertices {
            if p.iter().any(|v| !v.is_finite() || v.abs() > 1e9) {
                return Err(MeshCollisionError::NonFiniteOrExtremeCoordinate);
            }
            bounds.include(p);
        }
        let span = (0..3)
            .map(|d| bounds.high[d] - bounds.low[d])
            .fold(0.0_f64, f64::max);
        let magnitude = vertices
            .iter()
            .flatten()
            .copied()
            .map(f64::abs)
            .fold(0.0_f64, f64::max);
        let epsilon = (span * 1e-10)
            .max(magnitude * 64.0 * f64::EPSILON)
            .max(1e-15);
        if span < 1e-9 || span < 1000.0 * epsilon {
            return Err(MeshCollisionError::IllConditionedBounds);
        }
        let mut normals = reserve(triangles.len())?;
        for indices in &triangles {
            if indices.iter().any(|&i| i >= vertices.len()) {
                return Err(MeshCollisionError::InvalidIndex);
            }
            let [a, b, c] = indices.map(|i| vertices[i]);
            let ab = sub(b, a);
            let ac = sub(c, a);
            let n = cross(ab, ac);
            let area = dot(n, n).sqrt();
            let longest = dot(ab, ab)
                .max(dot(ac, ac))
                .max(dot(sub(c, b), sub(c, b)))
                .sqrt();
            if area <= epsilon * longest || !area.is_normal() {
                return Err(MeshCollisionError::DegenerateTriangle);
            }
            normals.push(n.map(|v| v / area));
        }
        let mut order = reserve(triangles.len())?;
        order.extend(0..triangles.len());
        let nodes = reserve(2 * triangles.len())?;
        let mut mesh = Self {
            vertices,
            triangles,
            normals,
            order,
            nodes,
            bounds,
            policy,
            epsilon,
        };
        if policy == MeshPolicy::ConvexSolid {
            mesh.validate_solid()?;
        }
        mesh.build_node(0, mesh.triangles.len());
        Ok(mesh)
    }
    pub fn vertices(&self) -> &[[f64; 3]] {
        &self.vertices
    }
    pub fn triangles(&self) -> &[[usize; 3]] {
        &self.triangles
    }
    pub fn bounds(&self) -> ([f64; 3], [f64; 3]) {
        (self.bounds.low, self.bounds.high)
    }
    pub fn policy(&self) -> MeshPolicy {
        self.policy
    }
    pub fn allocated_bytes(&self) -> usize {
        self.vertices.capacity() * std::mem::size_of::<[f64; 3]>()
            + self.triangles.capacity() * std::mem::size_of::<[usize; 3]>()
            + self.normals.capacity() * std::mem::size_of::<[f64; 3]>()
            + self.order.capacity() * std::mem::size_of::<usize>()
            + self.nodes.capacity() * std::mem::size_of::<Node>()
    }
    /// Inside or within the numerical contact tolerance of an admitted convex
    /// solid. Surface policy never labels any point as an interior.
    pub fn contains_or_touches(&self, point: [f64; 3]) -> bool {
        self.policy == MeshPolicy::ConvexSolid
            && point.iter().all(|v| v.is_finite())
            && (0..3).all(|d| {
                point[d] >= self.bounds.low[d] - self.epsilon
                    && point[d] <= self.bounds.high[d] + self.epsilon
            })
            && self
                .triangles
                .iter()
                .zip(&self.normals)
                .all(|(t, n)| dot(*n, sub(point, self.vertices[t[0]])) <= self.epsilon)
    }
    fn validate_solid(&mut self) -> Result<(), MeshCollisionError> {
        // Distinct indices at the same position are not a solid seam weld.
        // Require a combinatorial sphere with one closed fan at every vertex.
        let mut positions = HashSet::new();
        positions
            .try_reserve(self.vertices.len())
            .map_err(|_| MeshCollisionError::AllocationFailed)?;
        for v in &self.vertices {
            let key = v.map(|x| if x == 0.0 { 0 } else { x.to_bits() });
            if !positions.insert(key) {
                return Err(MeshCollisionError::NonManifoldVertexOrDuplicatePosition);
            }
        }
        let mut links = HashMap::new();
        links
            .try_reserve(3 * self.triangles.len())
            .map_err(|_| MeshCollisionError::AllocationFailed)?;
        let mut fans = reserve(self.vertices.len())?;
        fans.resize(self.vertices.len(), (usize::MAX, 0_usize));
        for &[a, b, c] in &self.triangles {
            for (center, from, to) in [(a, b, c), (b, c, a), (c, a, b)] {
                if links.insert((center, from), to).is_some() {
                    return Err(MeshCollisionError::NotWatertightOrConsistentlyWound);
                }
                fans[center].0 = from;
                fans[center].1 += 1;
            }
        }
        let mut edges = HashMap::new();
        edges
            .try_reserve(3 * self.triangles.len())
            .map_err(|_| MeshCollisionError::AllocationFailed)?;
        let mut parents = reserve(self.vertices.len())?;
        parents.extend(0..self.vertices.len());
        let root = |parents: &[usize], mut i: usize| {
            while parents[i] != i {
                i = parents[i];
            }
            i
        };
        let mut used = reserve(self.vertices.len())?;
        used.resize(self.vertices.len(), false);
        for t in &self.triangles {
            for (a, b) in [(t[0], t[1]), (t[1], t[2]), (t[2], t[0])] {
                used[a] = true;
                let ra = root(&parents, a);
                let rb = root(&parents, b);
                parents[ra] = rb;
                let entry = edges
                    .entry((a.min(b), a.max(b)))
                    .or_insert((0_usize, 0_i32));
                entry.0 += 1;
                entry.1 += if a < b { 1 } else { -1 };
            }
        }
        if edges
            .values()
            .any(|&(count, winding)| count != 2 || winding != 0)
        {
            return Err(MeshCollisionError::NotWatertightOrConsistentlyWound);
        }
        let first = root(&parents, 0);
        if (0..parents.len()).any(|i| !used[i] || root(&parents, i) != first) {
            return Err(MeshCollisionError::DisconnectedOrUnusedVertex);
        }
        if self.vertices.len() + self.triangles.len() != edges.len() + 2 {
            return Err(MeshCollisionError::NonManifoldVertexOrDuplicatePosition);
        }
        for (center, &(first, count)) in fans.iter().enumerate() {
            let mut current = first;
            for step in 0..count {
                current = *links
                    .get(&(center, current))
                    .ok_or(MeshCollisionError::NonManifoldVertexOrDuplicatePosition)?;
                if current == first && step + 1 != count {
                    return Err(MeshCollisionError::NonManifoldVertexOrDuplicatePosition);
                }
            }
            if current != first {
                return Err(MeshCollisionError::NonManifoldVertexOrDuplicatePosition);
            }
        }
        let centroid = std::array::from_fn(|d| {
            self.vertices
                .iter()
                .map(|v| v[d] / self.vertices.len() as f64)
                .sum()
        });
        let first_distance = dot(
            self.normals[0],
            sub(centroid, self.vertices[self.triangles[0][0]]),
        );
        if first_distance.abs() <= self.epsilon {
            return Err(MeshCollisionError::NotConvex);
        }
        if first_distance > 0.0 {
            for n in &mut self.normals {
                *n = n.map(|v| -v);
            }
        }
        for (t, n) in self.triangles.iter().zip(&self.normals) {
            let a = self.vertices[t[0]];
            if dot(*n, sub(centroid, a)) >= -self.epsilon
                || self
                    .vertices
                    .iter()
                    .any(|&v| dot(*n, sub(v, a)) > self.epsilon)
            {
                return Err(MeshCollisionError::NotConvex);
            }
        }
        Ok(())
    }
    fn build_node(&mut self, begin: usize, end: usize) -> usize {
        let mut bounds = Bounds::empty();
        for &i in &self.order[begin..end] {
            for v in self.triangles[i] {
                bounds.include(self.vertices[v]);
            }
        }
        let index = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            begin,
            end,
            children: None,
        });
        if end - begin > 8 {
            let axis = (0..3)
                .max_by(|&a, &b| {
                    (bounds.high[a] - bounds.low[a]).total_cmp(&(bounds.high[b] - bounds.low[b]))
                })
                .unwrap_or(0);
            let midpoint = begin + (end - begin) / 2;
            let vertices = &self.vertices;
            let triangles = &self.triangles;
            self.order[begin..end].select_nth_unstable_by(midpoint - begin, |&a, &b| {
                let centroid =
                    |i: usize| triangles[i].iter().map(|&v| vertices[v][axis]).sum::<f64>();
                centroid(a).total_cmp(&centroid(b)).then(a.cmp(&b))
            });
            let left = self.build_node(begin, midpoint);
            let right = self.build_node(midpoint, end);
            self.nodes[index].children = Some([left, right]);
        }
        index
    }
    fn triangle_hit(&self, index: usize, start: [f64; 3], end: [f64; 3]) -> Option<f64> {
        let [a, b, c] = self.triangles[index].map(|i| self.vertices[i]);
        let n = self.normals[index];
        let from = dot(n, sub(start, a));
        let to = dot(n, sub(end, a));
        let mut enter: f64 = 0.0;
        let mut exit: f64 = 1.0;
        // Intersect the segment with the complete tolerance prism: both sides
        // of the plane plus three edge half-planes. Testing only the first
        // plane-band point misses oblique segments entering through an edge.
        let mut clip = |first: f64, last: f64| {
            if first < 0.0 && last < 0.0 {
                return false;
            }
            if first < 0.0 {
                enter = enter.max(first / (first - last));
            }
            if last < 0.0 {
                exit = exit.min(first / (first - last));
            }
            enter <= exit
        };
        if !clip(self.epsilon - from, self.epsilon - to)
            || !clip(self.epsilon + from, self.epsilon + to)
        {
            return None;
        }
        // Solid normals may have been globally reversed for containment.
        let winding = cross(sub(b, a), sub(c, a));
        let wn = winding.map(|v| v / dot(winding, winding).sqrt());
        for (v, w) in [(a, b), (b, c), (c, a)] {
            let edge = sub(w, v);
            let tolerance = self.epsilon * dot(edge, edge).sqrt();
            if !clip(
                dot(cross(edge, sub(start, v)), wn) + tolerance,
                dot(cross(edge, sub(end, v)), wn) + tolerance,
            ) {
                return None;
            }
        }
        Some(enter)
    }
    /// Sweeps a point, stopping displacement before the earliest contact and
    /// removing normal velocity at a single plane. Simultaneous nonparallel
    /// contacts stop all velocity. Tangential displacement resumes next step;
    /// restitution, mesh friction/heat, finite-radius contact and force outputs
    /// are deliberately absent. A point initially on a two-sided surface has
    /// ambiguous side ownership and is conservatively stopped, even moving away.
    /// Nonfinite queries and starts inside/on a solid return an error. Queries
    /// require 64 machine epsilons at their coordinate/displacement scale to fit
    /// within the mesh contact tolerance; unresolvable huge sweeps are rejected.
    pub fn resolve_segment(
        &self,
        start: [f64; 3],
        end: [f64; 3],
        velocity: [f64; 3],
    ) -> Result<([f64; 3], [f64; 3]), MeshCollisionError> {
        if start
            .iter()
            .chain(&end)
            .chain(&velocity)
            .any(|v| !v.is_finite() || v.abs() > 1e100)
        {
            return Err(MeshCollisionError::NonFiniteQuery);
        }
        if self.contains_or_touches(start) {
            return Err(MeshCollisionError::QueryStartsInsideSolid);
        }
        let delta = sub(end, start);
        let length = dot(delta, delta).sqrt();
        let query_scale = start
            .iter()
            .chain(&end)
            .copied()
            .map(f64::abs)
            .fold(length, f64::max);
        if query_scale * 64.0 * f64::EPSILON > self.epsilon {
            return Err(MeshCollisionError::QueryPrecisionUnsupported);
        }
        if length == 0.0 {
            return Ok((end, velocity));
        }
        let mut earliest = f64::INFINITY;
        let mut normal = [0.0; 3];
        let mut corner = false;
        // Balanced binary subdivision of <=100000 leaves needs < 32 entries.
        let mut stack = [0_usize; 32];
        let mut size = 1;
        while size > 0 {
            size -= 1;
            let node = &self.nodes[stack[size]];
            if !node.bounds.intersects(start, end, self.epsilon) {
                continue;
            }
            if let Some([left, right]) = node.children {
                stack[size] = left;
                stack[size + 1] = right;
                size += 2;
            } else {
                for &i in &self.order[node.begin..node.end] {
                    if let Some(t) = self.triangle_hit(i, start, end) {
                        let tie = (t - earliest).abs() * length <= 4.0 * self.epsilon;
                        if t < earliest && !tie {
                            earliest = t;
                            normal = self.normals[i];
                            corner = false;
                        } else if tie {
                            earliest = earliest.min(t);
                            corner |= dot(normal, self.normals[i]).abs() < 1.0 - 1e-8;
                        }
                    }
                }
            }
        }
        if !earliest.is_finite() {
            return Ok((end, velocity));
        }
        // Clearance must be measured normal to the contact plane. Retreat by
        // Euclidean path length alone leaves grazing hits inside solid tolerance.
        let normal_travel = dot(delta, normal).abs();
        let retreat_scale = if normal_travel > 0.0 {
            normal_travel
        } else {
            length
        };
        let safe = (earliest - 4.0 * self.epsilon / retreat_scale).max(0.0);
        let position = std::array::from_fn(|d| start[d] + safe * delta[d]);
        let velocity = if corner {
            [0.0; 3]
        } else {
            let vn = dot(velocity, normal);
            std::array::from_fn(|d| velocity[d] - vn * normal[d])
        };
        Ok((position, velocity))
    }
}
