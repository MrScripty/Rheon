//! Bounded public-API reproduction of correction-budget boundary behavior.
#[path = "../tests/common/iteration_boundary_fixture.rs"]
mod fixture;
use fixture::*;
fn main() {
    for h in [0.05, 0.0015625] {
        for budget in [1, 2, 3, 7, 8, 512] {
            let mut o = owner(budget, false);
            let before = snapshot(&o);
            let result = o.step_extruded_with_forces(h, &[force(true)], |_| false);
            match result {
                Ok(r) => println!(
                    "{{\"case\":\"initial_reversed\",\"h\":{h:?},\"correction_budget\":{budget},\"status\":\"ACCEPTED\",\"reported_iterations\":{},\"equation_calls\":{},\"rate_norm\":{:?},\"unknowns\":{:?},\"velocity\":{:?},\"pressure\":{:?},\"time\":{:?},\"allocated_bytes\":{}}}",
                    r.step.planar.iterations,
                    r.step.planar.equation_evaluations,
                    r.step.planar.finite_momentum_rate_norm,
                    r.step.planar.unknowns,
                    o.state().velocity,
                    o.state().pressure_coefficients,
                    o.state().time,
                    o.allocated_bytes(),
                ),
                Err(error) => {
                    assert_eq!(before, snapshot(&o));
                    println!(
                        "{{\"case\":\"initial_reversed\",\"h\":{h:?},\"correction_budget\":{budget},\"status\":\"REFUSED\",\"error\":{:?},\"accepted_state_preserved\":true,\"allocated_bytes\":{}}}",
                        format!("{error:?}"),
                        o.allocated_bytes(),
                    );
                }
            }
        }
    }
    let mut o = owner(1, true);
    let r = o.step_extruded_with_forces(0.025, &[], |_| false).unwrap();
    println!(
        "{{\"case\":\"rest_seed\",\"status\":\"ACCEPTED\",\"reported_iterations\":{},\"equation_calls\":{},\"time\":{:?}}}",
        r.step.planar.iterations,
        r.step.planar.equation_evaluations,
        o.state().time,
    );
    let before = snapshot(&o);
    let result = o.step_extruded_with_forces(0.05, &[force(false)], |_| false);
    assert!(result.is_err());
    assert_eq!(before, snapshot(&o));
    println!(
        "{{\"case\":\"nonzero_accepted_one_update_limit\",\"status\":\"REFUSED\",\"error\":{:?},\"accepted_state_preserved\":true}}",
        format!("{:?}", result.unwrap_err()),
    );
}
