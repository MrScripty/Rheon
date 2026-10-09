# Opt-in spherical rigid mesh motion

Rheon already has immutable triangle surfaces, prescribed translational contact
queries and the qualified fixed-pose mesh impulse response. It had no pose/time
integrator. `SphericalRigidMotion` connects those retained types: load the actual
current world triangles through `TriangleMeshLoad`, kick a local `FrozenRigidBody`
proposal, then drift COM, orientation and time before a single publication.
Existing production fluid, collision predicates and PR34 response formulas stay
unchanged. Contact response, continuous swept rotation queries and fluid reaction
remain outside this capability. Endpoint meshes support the existing static
queries only. Open/duplicate facets retain the existing multiplicity semantics.

The original [Baraff rigid-body notes](https://www.cs.cmu.edu/~baraff/sigcourse/notesd1.pdf)
sections 2.9–2.11 supply momentum/inertia dynamics; section4 equations4–1 and4–2
supply the Hamilton product and world-angular-velocity quaternion convention.
The notes explicitly give the axis-angle quaternion and renormalization to
address numerical drift. The present bounded kick/drift connection is newly
derived here, not claimed as a discrete algorithm from that source. Continuum
mesh force/torque references and P1 quadrature premises remain in the preceding
[mesh-load](triangle-mesh-traction.md) and [impulse](rigid-mesh-impulse.md) derivations.

Initial supplied world geometry has identity body orientation. Retained reference
vertices are p_initial-COM_initial; this translation must be finite and round-trip
exactly to each supplied world vertex. No approximate inverse orientation or
silent snapping is used. COM and physical mass/inertia are caller-supplied and
need not be inferred from an open surface. Only exactly equal represented
positive principal moments (I,I,I) are supported. Spherical inertia remains
I_world=I*identity under rotation, so free-coast world omega is constant and no
anisotropic gyroscopic dynamics is being omitted within the admitted domain.

For positive requested h seconds, sample caller-supplied pressure(Pa) or world
traction(N/m²) on the CURRENT stamped mesh. PR34 applies J=hF and K=h*tau about
the CURRENT COM to actual velocities. Then c'=c+hV' metres, t'=t+h seconds, and
q'=dq*q, where q=[real,x,y,z] maps initial body coordinates into world coordinates,
theta=h|omega'| radians, and dq=[cos(theta/2),sin(theta/2)*omega'/|omega'|]. Zero
spin uses identity dq. Multiplication is on the LEFT because omega is in world
axes. Normalize q' by its actual computed norm. Every published vertex is
c'+q'[0,r]conjugate(q'), recomputed from the immutable initial reference r.

For an imposed instantaneous kick followed by spherical free coast, this drift
is exact over reals. If loads instead represent samples of a smooth time-varying
force/torque history, it is a first-order kick/drift splitting: it is not exact
finite-duration forced dynamics. For constant translational acceleration a,
V_n=V_0+nha and c_n=c_0+nhV_0+a*h²*n(n+1)/2; the continuous position differs by
+a*t*h/2. This explicit error is tested without widening tolerance. There is no
accuracy guarantee for discontinuous or under-resolved loads, no automatic
step selection, and no global convergence/stability claim.

A caller-selected finite positive max_dt_s and fixed theta<=0.25rad bound each
accepted step. The angular cap is a declared sampling/accuracy restriction, not
an enclosed stability certificate. Clock overflow or absorbed advancement
(t+h<=t) refuses. Requested h and actual represented elapsed time are reported
separately, with an unenclosed clock defect. Existing PR34 work/momentum
residuals remain unchanged, and translation/quaternion diagnostics are also
nearest rounded and unenclosed; they grant no conservation or stepping authority.
Lean proves finite exact-real displacement/time and quaternion algebra under
explicit premises. It does not prove the trigonometric library, compiler/IEEE
execution, swept geometry, full ODE solution or numerical order.

PR34 angular diagnostics describe spin about COM. For a fixed world origin,
total angular momentum is I*omega+c cross (mV). Under exact matched kick
increments its change is h*(tau_COM+c_old cross F); free drift contributes zero
because (hV_new) cross (mV_new)=0. New narrow Lean algebra states the endpoint
identity and its matched-impulse premises. The numerical lab separately checks
total world-origin angular increments on actual stored moving endpoints at the
unchanged absolute 1e-12 allowance; this is not an IEEE conservation proof.

Body and current-surface stamps must match before a scan. Every accepted step
increments body generation and surface version, including zero motion, so old
stamped inputs are refused. A late cancellation, geometry collapse or numerical
failure leaves velocities, COM, time, orientation, surface bytes/stamp and
payload accounting unchanged. Reference/current/proposed retained Vec capacity
payloads are capped; this excludes caller input creation, allocator metadata,
fixed headers and process RSS. Fallible reserves refuse allocation failures.
The exact admitted surface conditioning threshold is reused for every proposed
`TriangleSurface`; no acceptance tolerance is reset or relaxed.

Local replay emits all ACTUAL stored vertices, quaternion, COM, velocities,
clock and surface stamps. The moving JPEG85 uses these checked records directly;
it does not integrate a separate display-only motion. Generated outputs remain
outside Git. No held PR25/geometry-band campaign, hosted CI, main merge or deploy.
