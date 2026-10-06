# Observed native qualification

Actual Rust tests: 223 without default features, 229 default, 235 desktop; five focused release contracts. Formatting and three strict Clippy modes pass. Two actual 124-step native exports are byte-identical. Both Python modes replay all 124 steps and reject six meaningful corruptions, with byte-identical result packets.

| Field | h | Steps | Lumped velocity error | Geometry max error |
|---|---:|---:|---:|---:|
| initial | 0.05 | 2 | 3.44682422e-05 | 1.58577495e-06 |
| initial | 0.025 | 4 | 1.74328598e-05 | 7.68853020e-07 |
| initial | 0.0125 | 8 | 8.76668664e-06 | 3.77818142e-07 |
| initial | 0.00625 | 16 | 4.39602450e-06 | 1.87181875e-07 |
| initial | 0.003125 | 32 | 2.20120147e-06 | 9.31497413e-08 |
| pressure_state | 0.05 | 2 | 3.62420775e-05 | 1.57968490e-06 |
| pressure_state | 0.025 | 4 | 1.83378329e-05 | 7.64526168e-07 |
| pressure_state | 0.0125 | 8 | 9.22396439e-06 | 3.75357027e-07 |
| pressure_state | 0.00625 | 16 | 4.62577219e-06 | 1.85879319e-07 |
| pressure_state | 0.003125 | 32 | 2.31627342e-06 | 9.24807528e-08 |

Independent accepted-state replay maxima:

```json
{
  "momentum": 9.847806936090065e-14,
  "direct_momentum": 1.0408168092534077e-13,
  "work_ratio": 0.015370244416953706,
  "ledger_ratio": 0.004342912941601095,
  "gcl_ratio": 0.01890420446818611,
  "quad": 4.7704895589362195e-18,
  "velocity_difference": 3.774758283725532e-15,
  "pressure_difference": 1.1622647289044608e-16
}
```

The native storage budget is 450,320 bytes including conservative bounded scratch reservation; it excludes process RSS, allocator metadata and an ABI stack theorem. All original Rust fixtures/tests/examples and Cargo inputs remain unchanged except the two documented integration/owner initialization files. Six cancellation stages preserve every accepted bit after a nonzero state and continuation matches an uninterrupted owner. Invalid scale intervals, bounded iteration failure, memory shortage and unsupported fields reject.

The native render uses 64 actual accepted endpoint records with exported PS geometry, full velocity, mass and physical pressure. Endpoint-donor momentum remains first order and differs from actual varying-donor path momentum. General legacy liquid advancement, continuum traction/spatial convergence, all-real native path/sign and IEEE/mesh-family proofs, positivity, variable density, adhesion/capillarity and reconstruction remain unqualified. No new Lean theorem is claimed.
