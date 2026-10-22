#!/usr/bin/env python3
"""Prospective CV3-A independent analyzer. No selected work on import.

analyze.py audit NATIVE_CASE FRESH_OUTPUT
analyze.py case NATIVE_CASE FRESH_OUTPUT
analyze.py aggregate NATIVE_ROOT CASE_REPORT_ROOT FRESH_OUTPUT

Execution requires the parent's separate source/protocol/resource freeze. Each
expensive case invocation belongs under the frozen external resource supervisor.
"""
from fractions import Fraction as F
import hashlib
import json
from pathlib import Path
import sys
import exact_operators as eo
from native_csv import read_case, require, hashes, cases, GATE
from audit_native import audit, Gates
from validated import (Budget, solve_certified, json_exact, finest_gate, order_gate,
                       observed_order_interval)
from bounded_norms import Subdivision
from continuum import fields
from physical_norms import norm_metrics, normalized_algebraic_radii, deciding_intervals
from spectrum import certified_spectrum
from cross_phase import compare


def write_json(path, value):
    Path(path).write_text(json.dumps(json_exact(value),sort_keys=True,separators=(',',':'))+'\n')


def parse_exact(value):
    if isinstance(value,dict):
        if set(value) == {'numerator','denominator'}:
            require(all(isinstance(value[k],str) and len(value[k]) <= 1000000 for k in value), 'bounded exact report decimal strings')
            n,d = int(value['numerator']),int(value['denominator'])
            require(d > 0 and str(n) == value['numerator'] and str(d) == value['denominator'], 'canonical exact report rational')
            require(n.bit_length() <= 3000000 and d.bit_length() <= 3000000, 'bounded exact report integer bits')
            return F(n,d)
        return {k:parse_exact(v) for k,v in value.items()}
    if isinstance(value,list):
        return [parse_exact(v) for v in value]
    return value


def read_json(path):
    def unique(pairs):
        result = {}
        for k,v in pairs:
            require(k not in result, 'duplicate JSON report key'); result[k] = v
        return result
    return parse_exact(json.loads(Path(path).read_text(),object_pairs_hook=unique,
                      parse_constant=lambda s:(_ for _ in ()).throw(RuntimeError('CV3 input guard: nonfinite report'))))


def fresh(path, raw):
    path,raw = Path(path),Path(raw)
    require(not path.exists() and not path.resolve().is_relative_to(raw.resolve()), 'fresh independent output outside native tree')
    path.mkdir(parents=True)
    return path


def persist_generator(out, target, certificate):
    generator = certificate.pop('inverse_generator')
    payload = {'target':target,'precision_digits':120,'encoding':'exact rational finite binary mpf values',
               'inverse':generator,'center':certificate['center']}
    path = out/(target+'-generator.json'); write_json(path,payload)
    certificate['generator_sha256'] = hashlib.sha256(path.read_bytes()).hexdigest()
    certificate['generator_filename'] = path.name


def reference_gate(gates, target, native, certificate, nv):
    center,eta = certificate['center'],certificate['eta']
    u_error = max((abs(a-b) for a,b in zip(native[:nv],center[:nv])),default=F(0))+eta
    p_error = max((abs(a-b) for a,b in zip(native[nv:],center[nv:])),default=F(0))+eta
    gates.record('solution-'+target+'-velocity',u_error)
    gates.record('solution-'+target+'-pressure',p_error/(1+F.from_float(.1)))
    return {'velocity':u_error,'pressure':p_error}


def case_report(raw,out,mode):
    budget = Budget(1800); data = read_case(raw)
    if 'native_refused' in data:
        report = {'mode':mode,'case':data['name'],'status':'native-refused',
                  'gates':{'native-completion':{'value':F(1),'limit':GATE,'pass':False}},
                  'native_refusal':data['native_refused'],'input_sha256':data['input_sha256'],
                  'all_required_gates_pass':False}
        write_json(out/'analysis.json',report); return False
    op,loads,gates,observations = audit(data,budget)
    report = {'mode':mode,'case':data['name'],'input_sha256':data['input_sha256'],
              'gates':gates.values,'physical_observations':observations,
              'exact_rank':op['rank'],'rank_pivots':op['rank_pivots'],
              'velocity_coefficients':data['nv'],'pressure_rows':data['nc']}
    write_json(out/'progress.json',dict(report,stage='exact native audit completed'))
    if mode == 'case':
        nv,nc = data['nv'],data['nc']; a,b = data['factors']['A'],data['native']['B']
        c1 = [a[i]+[-b[c][i] for c in range(nc)] for i in range(nv)]+[[-x for x in row]+[F(0)]*nc for row in b]
        rhs1 = data['loads']['rhs']+[F(0)]*nc
        native = [data['states'][3,'u'][i][0] for i in range(nv)]+[data['states'][3,'p'][i][0] for i in range(nc)]
        references = {}; radii = {}
        for target,c,rhs in (('R1',c1,rhs1),('R2',eo.system(op,F(1)),loads['rhs'])):
            certificate = solve_certified(c,rhs,budget)
            radii[target] = reference_gate(gates,target,native,certificate,nv)
            persist_generator(out,target,certificate)
            references[target] = certificate
            write_json(out/'progress.json',dict(report,stage=target+' exact certificate completed',references=references))
        c1center,c2center = references['R1']['center'],references['R2']['center']
        reference_radius = references['R1']['eta']+references['R2']['eta']
        gap_u = max(abs(a-b) for a,b in zip(c1center[:nv],c2center[:nv]))
        gap_p = max(abs(a-b) for a,b in zip(c1center[nv:],c2center[nv:]))
        pref = 1+F.from_float(.1)
        target_gap = {'velocity':(max(F(0),gap_u-reference_radius),gap_u+reference_radius),
                      'pressure':(max(F(0),gap_p-reference_radius)/pref,(gap_p+reference_radius)/pref),
                      'meaning':'original-coordinate rounded-export versus exact-integral solution gap; pressure scaled by physical Pref'}
        iteration_errors = {}
        for target,certificate in references.items():
            iteration_errors[target] = {}
            for it in range(4):
                values = [data['states'][it,'u'][i][0] for i in range(nv)]+[data['states'][it,'p'][i][0] for i in range(nc)]
                du = max(abs(a-b) for a,b in zip(values[:nv],certificate['center'][:nv]))
                dp = max(abs(a-b) for a,b in zip(values[nv:],certificate['center'][nv:]))
                eta = certificate['eta']
                iteration_errors[target][str(it)] = {'velocity':(max(F(0),du-eta),du+eta),
                                                    'pressure':(max(F(0),dp-eta)/pref,(dp+eta)/pref)}
        ledger = Subdivision(budget)
        measured = norm_metrics(op,native[:nv],native[nv:],loads['fields'],ledger)
        measured.pop('reconstructed')
        algebraic_radii = normalized_algebraic_radii(op,radii['R2'],measured)
        intervals = deciding_intervals(measured,algebraic_radii)
        uncertainty = {}
        for field in ('velocity','pressure','traction'):
            uncertainty[field] = {}
            for norm in ('L1','L2'):
                base = measured['metrics'][field][norm]
                # Full integration interval width, not zero or merely printed
                # precision; algebraic/assembly target radius also propagated.
                total = base[1]-base[0]+algebraic_radii[field][norm]
                passed = intervals[field][norm][0] > 0 and total <= intervals[field][norm][0]/100
                uncertainty[field][norm] = {'radius':total,'pass':passed}
                gates.record('one-percent-error-'+field+'-'+norm,
                             total/intervals[field][norm][0] if intervals[field][norm][0] > 0 else F(1),F(1,100))
        spectrum = certified_spectrum(op,budget)
        generator = {'inverse':spectrum.pop('inverse_generator'),
                     'eigenvalues':spectrum.pop('spectral_generator_values'),
                     'eigenvectors':spectrum.pop('spectral_generator_vectors'),
                     'precision_digits':120,'encoding':'exact rational mpf/f64 generator values'}
        spath = out/'H1-spectral-generator.json'; write_json(spath,generator)
        spectrum['generator_sha256'] = hashlib.sha256(spath.read_bytes()).hexdigest()
        spectrum['generator_filename'] = spath.name
        report.update(references=references,physical_norms=measured,deciding_intervals=intervals,
                      rounded_vs_integral_solution_gap=target_gap,iteration_solution_errors=iteration_errors,
                      total_error_uncertainty=uncertainty,algebraic_norm_radii=algebraic_radii,
                      coefficient_reference_radii=radii,
                      h1_spectrum=spectrum,native_solution={'u':native[:nv],'p':native[nv:]})
    require(hashes(raw) == data['input_sha256'], 'native input bytes changed during independent analysis')
    report.update(status='numerical-pass' if gates.passed() else 'numerical-fail',
                  gates=gates.values,all_required_gates_pass=gates.passed())
    write_json(out/'analysis.json',report)
    return gates.passed()


def aggregate(raw,reports,out):
    budget = Budget(1800); raw,reports = Path(raw),Path(reports)
    expected = cases(); require({p.name for p in raw.iterdir() if p.is_dir()} == set(expected), 'complete nine-case native corpus')
    require({p.name for p in reports.iterdir() if p.is_dir()} == set(expected), 'complete nine-case report corpus')
    all_rows = {}; all_ok = True
    for name in expected:
        row = read_json(reports/name/'analysis.json')
        require(row.get('mode') == 'case' and row.get('case') == name, 'bound full scientific case report')
        require(isinstance(row.get('all_required_gates_pass'),bool) and isinstance(row.get('gates'),dict) and row['gates'], 'complete typed case acceptance gates')
        for gate in row['gates'].values():
            require(set(gate) == {'value','limit','pass'} and isinstance(gate['value'],F) and isinstance(gate['limit'],F)
                    and isinstance(gate['pass'],bool) and gate['value'] >= 0 and gate['limit'] > 0
                    and gate['pass'] == (gate['value'] <= gate['limit']), 'exact case gate predicate')
        require(row['all_required_gates_pass'] == all(gate['pass'] for gate in row['gates'].values()), 'case required-gate consistency')
        require(row.get('input_sha256') == hashes(raw/name), 'bound report native input hashes')
        if row.get('status') == 'native-refused':
            all_ok = False
        else:
            require('deciding_intervals' in row and 'h1_spectrum' in row and set(row['references']) == {'R1','R2'}, 'complete scientific case report')
            for target,certificate in row['references'].items():
                filename = target+'-generator.json'
                require(certificate['generator_filename'] == filename and hashlib.sha256((reports/name/filename).read_bytes()).hexdigest() == certificate['generator_sha256'], 'bound retained inverse generator')
            spectrum = row['h1_spectrum']; filename = 'H1-spectral-generator.json'
            require(spectrum['generator_filename'] == filename and hashlib.sha256((reports/name/filename).read_bytes()).hexdigest() == spectrum['generator_sha256'], 'bound retained spectral generator')
            all_ok = bool(row['all_required_gates_pass']) and all_ok
        all_rows[name] = row
    decisions = {}; gaps = {}; phase = {}; summary_gates = Gates()
    for name,row in all_rows.items():
        require(isinstance(row['all_required_gates_pass'],bool), 'boolean case acceptance')
        summary_gates.record('case-required:'+name,F(0) if row['all_required_gates_pass'] else F(1))
    for family in ('rectangle-origin','square-17-origin','square-17-quarter'):
        decisions[family] = {}; gaps[family] = {}
        for field in ('velocity','pressure','traction'):
            for norm in ('L1','L2'):
                key = field+'-'+norm; rows = [all_rows[f'{family}-n{n}'] for n in (4,8,16)]
                if any('deciding_intervals' not in row for row in rows):
                    decisions[family][key] = {'pass':False,'reason':'incomplete/refused scientific levels'}
                    summary_gates.record('complete-levels:'+family+':'+key,F(1)); all_ok = False; continue
                intervals = [row['deciding_intervals'][field][norm] for row in rows]
                family_gaps = []
                for k in (0,1):
                    coarse,fine = intervals[k],intervals[k+1]
                    lo,hi = coarse[0]-fine[1],coarse[1]-fine[0]
                    radius = rows[k]['total_error_uncertainty'][field][norm]['radius']+rows[k+1]['total_error_uncertainty'][field][norm]['radius']
                    distance = lo if lo > 0 else -hi if hi < 0 else F(0)
                    gap_ok = distance > 0 and radius <= distance/100
                    family_gaps.append({'difference_interval':(lo,hi),'total_uncertainty':radius,
                                        'one_percent_pass':gap_ok,'order_at_least_point9':order_gate(coarse,fine),
                                        'observed_order_enclosure':observed_order_interval(coarse,fine)})
                    summary_gates.record('gap-uncertainty:'+family+':'+key+':'+('4-8' if k == 0 else '8-16'),
                                         radius/distance if distance > 0 else F(1),F(1,100))
                finest_ok = finest_gate(intervals[2]); order_ok = family_gaps[1]['order_at_least_point9']; gap_ok = family_gaps[1]['one_percent_pass']
                decision = finest_ok and order_ok and gap_ok and family_gaps[0]['one_percent_pass']
                decisions[family][key] = {'finest_5percent_pass':finest_ok,'order_8_16_pass':order_ok,
                                         'gap_4_8_uncertainty_pass':family_gaps[0]['one_percent_pass'],
                                         'gap_8_16_uncertainty_pass':gap_ok,'pass':decision}
                summary_gates.record('finest-five-percent:'+family+':'+key,intervals[2][1],F(1,20))
                summary_gates.record('order-eight-sixteen:'+family+':'+key,F(0) if order_ok else F(1))
                gaps[family][key] = family_gaps; all_ok = decision and all_ok
    for n in (4,8,16):
        names = [f'square-17-{p}-n{n}' for p in ('origin','quarter')]
        if any('native_solution' not in all_rows[name] for name in names):
            phase[str(n)] = {'status':'unavailable after retained refusal'}; all_ok = False; continue
        data = [read_case(raw/name) for name in names]
        ops = [eo.build(d['source'],d['n'],d['origin'],F(1),F(1),F.from_float(.1)) for d in data]
        solutions = [all_rows[name]['native_solution'] for name in names]
        phase[str(n)] = compare(ops[0],solutions[0]['u'],solutions[0]['p'],ops[1],solutions[1]['u'],solutions[1]['p'],
                                fields(F.from_float(.1)),Subdivision(budget))
        combined_radius = {kind:sum((all_rows[name]['coefficient_reference_radii']['R2'][kind] for name in names),F(0))
                           for kind in ('velocity','pressure')}
        radii = normalized_algebraic_radii(ops[0],combined_radius,phase[str(n)])
        phase[str(n)]['propagated_R2_norm_radii'] = radii
        phase[str(n)]['propagated_difference_intervals'] = deciding_intervals(phase[str(n)],radii)
    require(all_ok == summary_gates.passed(), 'aggregate required-gate consistency')
    report = {'mode':'aggregate','status':'numerical-pass' if all_ok else 'numerical-fail',
              'corpus':list(expected),'gates':summary_gates.values,'case_reports':all_rows,
              'refinement_decisions':decisions,'refinement_differences':gaps,
              'cross_phase':phase,'all_required_gates_pass':all_ok,
              'claim':'bounded static prescribed-traction polygon MMS only; no production or uniform stability admission'}
    write_json(out/'analysis.json',report)
    return all_ok


def main(argv=None):
    # Scoped process policy, not import-time mutation and never unlimited.
    sys.set_int_max_str_digits(1000000)
    argv = sys.argv[1:] if argv is None else argv
    require(len(argv) in (3,4), 'usage audit/case INPUT OUTPUT or aggregate INPUT REPORTS OUTPUT')
    mode = argv[0]
    require((mode in ('audit','case') and len(argv) == 3) or (mode == 'aggregate' and len(argv) == 4), 'declared analyzer command')
    out = fresh(argv[-1],argv[1])
    try:
        passed = aggregate(argv[1],argv[2],out) if mode == 'aggregate' else case_report(Path(argv[1]),out,mode)
    except BaseException as error:
        write_json(out/'refusal.json',{'mode':mode,'status':'refused','reason':str(error),'all_required_gates_pass':False})
        raise
    return 0 if passed else 1


if __name__ == '__main__':
    sys.exit(main())
