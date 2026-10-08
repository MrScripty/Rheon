"""Three rejected quadratic cells; failure refinement is not liquid accuracy."""
import json
from cell import solve_cell


def study():
    cells=[solve_cell(h)for h in [.05,.025,.0125]]
    return dict(scope='rejected coupled temporal cells with sampled velocity/pressure and separate weighted work',
                cells=cells,work_defect_refinement_ratios=[abs(cells[i]['actual_momentum_residual_work']/cells[i+1]['actual_momentum_residual_work'])for i in range(2)],
                all_fixed_work_gates_fail=all(not c['work_gate_pass']for c in cells),accepted_advancing_steps=0,
                continuum_or_accepted_temporal_accuracy_claim=False)


if __name__=='__main__':print(json.dumps(study(),indent=2))
