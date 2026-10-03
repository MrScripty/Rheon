# A Algorithm reference

These algorithms specify future implementation behavior. The numerical companion executes a smaller independent subset. The pseudocode is not a tested Rust solver. Each routine names ownership and failure boundaries.

## Validated grid construction

Inputs are positive counts, finite positive spacing, finite origin and caller-supplied memory limit. Compute all capacities and bytes with checked arithmetic. Reject overflow, unsupported dimensions or budget excess before allocation. Allocate all required buffers before publishing a state.

~~~
make_grid(nx, ny, nz, spacing, origin, memory_limit):
    require finite(spacing) and spacing > 0
    require finite(origin) and positive counts
    cells = checked_product(nx, ny, nz)
    xfaces = checked_product(checked_add(nx, 1), ny, nz)
    yfaces = checked_product(nx, checked_add(ny, 1), nz)
    zfaces = checked_product(nx, ny, checked_add(nz, 1))
    bytes = checked_layout_bytes(cells, xfaces, yfaces, zfaces)
    if bytes > memory_limit: return ResourceLimit
    allocate state and workspace or return AllocationFailure
    return ValidatedGrid
~~~

The indexing theorem applies after finite-range checks. Each view retains its shape so face arrays cannot accidentally use cell strides.

## Matrix free pressure

For each pressure cell gather incident faces with nonnegative coefficients. Closed walls contribute known flux to the right-hand side. Prescribed pressure is eliminated consistently.

~~~
apply_A(p, out, topology):
    for each pressure cell i:
        total = 0
        for neighbor j across internal face e:
            total += weight[e] * (p[i] - p[j])
        for each prescribed-pressure face e:
            total += boundary_diagonal[e] * p[i]
        out[i] = total
~~~

This gives positive diagonals and nonpositive off-diagonals. Known boundary pressure adds a matching right-hand-side term. A pinned pressure needs matching row and column elimination.

## Pressure solve lifecycle

The preconditioner is assumed SPD by a separately justified construction. Check initial residual before any quotient. Final acceptance uses a freshly computed residual.

~~~
solve(A, M, b, p, tolerances, iteration_limit):
    r = b - A(p)
    if not finite(r): return NonfiniteState
    if accepted(r, b): return Converged
    z = M_inverse(r)
    d = z
    rz = dot(r, z)
    for k in 0 .. iteration_limit:
        Ad = A(d)
        curvature = dot(d, Ad)
        if not finite(curvature) or curvature <= 0:
            return Breakdown
        alpha = rz / curvature
        p += alpha * d
        r -= alpha * Ad
        if candidate_tolerance_met(r):
            true_r = b - A(p)
            if accepted(true_r, b): return Converged
            r = true_r
            z = M_inverse(r)
            d = z
            rz = dot(r, z)
            continue
        z = M_inverse(r)
        next_rz = dot(r, z)
        if not finite(next_rz) or rz <= 0:
            return Breakdown
        beta = next_rz / rz
        d = z + beta * d
        rz = next_rz
    return IterationLimit(p, b - A(p))
~~~

Production also validates updated vectors and supports cancellation. Tiny negative curvature still indicates breakdown of expected SPD arithmetic; precision escalation or retry must be explicit. The stopping function combines absolute and relative scales, then verifies physical divergence.

## A fixed boundary substep

This first-order proposal holds geometry fixed over a substep. Reordering requires validation.

~~~
step(accepted, boundary_snapshot, requested_dt):
    validate configuration and state version
    choose dt and report its limiting reason
    initialize candidate buffers
    advect velocity into separate face arrays
    apply declared body accelerations
    apply viscosity if enabled
    impose prescribed normal wall velocities
    build pressure topology and component gauges
    build rhs through the same divergence operator
    solve pressure or return a declared failure
    subtract the matching pressure gradient
    verify boundary consistency
    recompute divergence and finite-state diagnostics
    transport scalars at the declared velocity time level
    integrate sources with explicit amounts
    validate field bounds and diagnostics
    publish candidate atomically at time + dt
~~~

Liquids need separately derived interface transport, classification, velocity extension and free-surface pressure stages. Adding a surface renderer does not turn this into a liquid algorithm.

## Conservative amounts

Compute each shared face flux once from immutable old state. A gather update reads a flux array without races:

~~~
for each face e:
    flux[e] = selected_flux(old, velocity, geometry, e)
for each cell i:
    amount_next[i] = amount_old[i]
    for incident face e:
        amount_next[i] += dt * signed_inflow(i, e) * flux[e]
    amount_next[i] += integrated_source[i]
~~~

The conserved quantity is amount, not concentration. Divide by cell volume only when constructing concentration. Moving cells require geometric transport terms absent from this static-cell argument.

## Diagnostics

A packet should contain state ID, boundary version, start/end time, accepted substeps, limiting reason, maximum and RMS divergence, true residual, iterations, total tracer or liquid amount, kinetic-energy estimate, allocated bytes and termination reason. Define units and normalization. Missing measurements are unavailable, not zero.
