#!/usr/bin/env python3
"""ONE separately approved physical roster, never called by source/unit checks.

No default execution. Uses a frozen binary/source/input, one primal solve per
level, sequential180s limits, no retries/extra levels or reference input. Reports
physical comparisons only after each native process exits. Never sets a physical
qualification flag: propagated source/goal and spatial error remain separate.
"""
import argparse,hashlib,json,math,subprocess,sys,time
from pathlib import Path
from check_flat_wall_ritz import compare,require,MAX_BYTES,FILE_CAP
from flat_wall_force_source import write as write_source

def main():
    p=argparse.ArgumentParser();p.add_argument('--approve-one-shot-physical',action='store_true')
    p.add_argument('--binary',type=Path,required=True);p.add_argument('--binary-sha256',required=True)
    p.add_argument('--head',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
    require(a.approve_one_shot_physical,'explicit one-shot physical campaign approval required')
    repo=Path(__file__).resolve().parents[1];head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
    require(head==a.head and not subprocess.check_output(['git','status','--porcelain=v1'],cwd=repo,text=True),'source must be frozen clean exact head')
    require(hashlib.sha256(a.binary.read_bytes()).hexdigest()==a.binary_sha256,'frozen binary hash mismatch')
    out=a.output.resolve();require(not out.is_relative_to(repo),'generated output outside Git required');out.mkdir(parents=True,exist_ok=False)
    source=out/'body-force.csv';source_receipt=write_source(source);runs=[];begin=time.monotonic()
    for n in (6,9,12):
        level=out/f'n{n}';log=out/f'n{n}.log'
        with log.open('wb') as stream:
            child=subprocess.run([str(a.binary.resolve()),'--physical',str(n),str(source),str(level)],stdout=stream,stderr=stream,timeout=180)
        require(child.returncode==0,f'N{n} refused; no retry or changed roster')
        require(sum(x.stat().st_size for x in level.iterdir())<=FILE_CAP,'native level file cap')
        # Native provider has EXITED before exact continuum reference is used.
        comparison_file=out/f'n{n}-comparison.json'
        with (out/f'n{n}-comparison.log').open('wb') as stream:
            child=subprocess.run([sys.executable,str(repo/'tools/flat_wall_compare_worker.py'),'--physical-records','--records',str(level/'records.jsonl'),'--source',str(source),'--head',head,'--binary',str(a.binary.resolve()),'--output',str(comparison_file)],stdout=stream,stderr=stream,timeout=180)
        require(child.returncode==0,f'N{n} exact comparison refused; no retry')
        comparison=json.loads(comparison_file.read_text())
        reference_force=[-1.,0.,0.];reference_torque=[0.,-1./60.,-.5]
        measurements=[]
        with (level/'records.jsonl').open() as stream:
            for line in stream:
                r=json.loads(line)
                if r['kind']=='traction':
                    r['force_error_N']=[x-y for x,y in zip(r['force'],reference_force)]
                    r['torque_error_Nm']=[x-y for x,y in zip(r['torque'],reference_torque)]
                    r['force_error_norm_N']=math.sqrt(sum(x*x for x in r['force_error_N']))
                    r['torque_error_norm_Nm']=math.sqrt(sum(x*x for x in r['torque_error_Nm']))
                    measurements.append(r)
        runs.append({'n':n,'comparison':comparison,'measurements':measurements})
        require(hashlib.sha256(a.binary.read_bytes()).hexdigest()==a.binary_sha256,'binary changed during campaign')
        require(subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==head and not subprocess.check_output(['git','status','--porcelain=v1'],cwd=repo,text=True),'source changed during campaign')
    trends={}
    for scheme in ('p1','normal_p2'):
        samples=[next(r for r in level['measurements'] if r['scheme']==scheme) for level in runs]
        errors=[[r[key] for r in samples] for key in ('force_error_norm_N','torque_error_norm_Nm')]
        trends[scheme]={'decreasing_force_and_torque_errors':all(all(a>b for a,b in zip(e,e[1:])) for e in errors),
            'adjacent_force_orders':[math.log(errors[0][i]/errors[0][i+1])/math.log((6,9,12)[i+1]/(6,9,12)[i]) if errors[0][i]>0 and errors[0][i+1]>0 else None for i in (0,1)],
            'adjacent_torque_orders':[math.log(errors[1][i]/errors[1][i+1])/math.log((6,9,12)[i+1]/(6,9,12)[i]) if errors[1][i]>0 and errors[1][i+1]>0 else None for i in (0,1)]}
    report={'head':head,'binary_sha256':a.binary_sha256,'source':source_receipt,'roster':[6,9,12],
        'reference_force_N':[-1,0,0],'reference_torque_Nm':['0','-1/60','-1/2'],'runs':runs,'trends':trends,
        'physical_qualified':False,'convergence_claim':False,'pressure_or_stepping_executed':False,
        'remaining_acceptance':'Load/source/goal/spatial error and absolute application budgets remain unqualified; a three-level trend alone cannot set qualification.',
        'comparison_managed_cap':MAX_BYTES,'native_limit_seconds_per_level':180,'comparison_limit_seconds_per_level':180,'elapsed_seconds':time.monotonic()-begin}
    text=json.dumps(report,indent=2)+'\n';require(len(text.encode())<=FILE_CAP,'summary output cap')
    (out/'campaign-report.json').write_text(text)
    require(sum(x.stat().st_size for x in out.rglob('*') if x.is_file())<=4*FILE_CAP,'roster output cap')
    print(json.dumps({'report':str(out/'campaign-report.json'),'physical_qualified':False,'convergence_claim':False}))
if __name__=='__main__':main()
