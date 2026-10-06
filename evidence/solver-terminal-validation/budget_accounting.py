"""Explicit counters from actual trace; distinguish Newton and chart solves."""
from pathlib import Path
import importlib.util,json
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('actual_audit',P/'audit_trace.py');audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
rows=[]
for stem,budgets in [('trace-boundary',[1,2,3,7,8,512]*2+[1,1]),('trace-finer',None)]:
 events=[json.loads(l)for l in(P/(stem+'-stderr.log')).read_text().splitlines()];attempts=audit.attempts(events)
 for i,a in enumerate(attempts):
  budget=7 if budgets is None else budgets[i];passed,_,calls=audit.validate(a,budget)
  rows.append({'stream':stem,'case':a['case'],'retry':a['retry'],'configured_budget':budget,'authorized_correction_cap':min(budget,7),'computed_Newton_correction_solves':len(a['corrections']),'unperturbed_Newton_residual_checks':len(a['checks'])+len(a['terminal']),'counted_Newton_equation_evaluations':calls,'separate_qualification_equation_evaluations':2 if passed else 0,'total_work_equation_evaluations_excluding_seed_and_diagnostic_observations':calls+(2 if passed else 0),'Newton_gate_passed':passed})
answer={'source':'922466bd9efdf4612462288fbe2df5a679ef4ccb','original_maximum_unperturbed_residual_checks':7,'candidate_maximum_unperturbed_residual_checks':max(r['unperturbed_Newton_residual_checks']for r in rows),'maximum_computed_Newton_correction_solves':max(r['computed_Newton_correction_solves']for r in rows),'maximum_counted_Newton_equation_evaluations':max(r['counted_Newton_equation_evaluations']for r in rows),'maximum_accepted_total_work_equation_evaluations_excluding_seed':max(r['total_work_equation_evaluations_excluding_seed_and_diagnostic_observations']for r in rows if r['Newton_gate_passed']),'unchanged_Newton_gate':1e-13,'unchanged_reported_Newton_equation_budget':200,'reported_iterations_remain_loop_passes':True,'additional_Newton_correction_solve_permitted':False,'equation_evaluation_performs_existing_15_by_15_chart_solves':True,'seed_assembly_and_diagnostic_observations_are_excluded_from_these_work_equation_totals':True,'older_documentation_budget_ambiguity_preserved':True,'attempts':rows}
if (answer['candidate_maximum_unperturbed_residual_checks'],answer['maximum_computed_Newton_correction_solves'],answer['maximum_counted_Newton_equation_evaluations'],answer['maximum_accepted_total_work_equation_evaluations_excluding_seed'])!=(8,7,50,52):raise ValueError('unexpected actual correction/check/equation accounting')
print(json.dumps(answer,indent=2))
