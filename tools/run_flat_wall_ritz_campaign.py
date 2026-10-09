#!/usr/bin/env python3
"""One separately approved fixed physical roster, guarded before any Git work.

No physical qualification flags, retries, extra levels or reference solver input.
The supervisor enforces the aggregate deadline, sampled RSS threshold and its
own bounded logs; fixed pre-write producer reservations sum to exactly4MiB.
"""
import sys,time
ENTRY_TIME=time.monotonic()
sys.dont_write_bytecode=True
import argparse,json,math,os
from pathlib import Path
from flat_wall_campaign_guard import (supervise,controller_deadline,run_bounded,
    write_reserved,SOURCE_CAP,COMPARISON_CAP,SUMMARY_CAP,NATIVE_CAP,QUOTAS,
    TOTAL_SECONDS,CLEANUP_SECONDS,SAMPLE_SECONDS,RSS_THRESHOLD,OUTPUT_CAP)

def arguments(argv):
    p=argparse.ArgumentParser();p.add_argument('--approve-one-shot-physical',action='store_true')
    p.add_argument('--binary',type=Path,required=True);p.add_argument('--binary-sha256',required=True)
    p.add_argument('--head',required=True);p.add_argument('--output',type=Path,required=True)
    a=p.parse_args(argv)
    if not a.approve_one_shot_physical:raise ValueError('explicit one-shot physical campaign approval required')
    if sum(len(x.encode()) for x in argv)>8192:raise ValueError('argument payload cap')
    return a

def prepare(argv):
    a=arguments(argv);repo=Path(__file__).resolve().parents[1];out=a.output.resolve()
    if out.is_relative_to(repo):raise ValueError('generated output outside Git required')
    if not out.parent.is_dir() or out.exists():raise ValueError('new external output directory required')
    # -B also prevents unbudgeted bytecode writes by the controller and workers.
    return [sys.executable,'-B',str(Path(__file__).resolve()),'--guard-controller',*argv],out

def controller(argv):
    deadline=controller_deadline();a=arguments(argv)
    from check_flat_wall_ritz import file_digest,require,MAX_BYTES,FILE_CAP
    from flat_wall_force_source import write as write_source
    repo=Path(__file__).resolve().parents[1];out=a.output.resolve();launches=[]
    require(not (repo/'.gitmodules').exists(),'submodule helper processes unsupported')
    def git(*args):
        launches.append({'role':'git','command':list(args),'seconds':10})
        # Disable optional Git helper invocation and index writes for this fixed
        # read-only roster. The environment/security configuration is unchanged.
        command=['git','--no-optional-locks','-c','core.fsmonitor=false','-c','core.untrackedCache=false',
                 '-c','submodule.recurse=false',*args]
        return run_bounded(command,deadline=deadline,seconds=10,cwd=repo).decode('ascii').strip()
    head=git('rev-parse','HEAD')
    require(head==a.head and not git('status','--porcelain=v1','--ignore-submodules=all'),'source must be frozen clean exact head')
    require(file_digest(a.binary)==a.binary_sha256,'frozen binary hash mismatch')
    require(out.is_dir() and not out.is_relative_to(repo),'supervisor output mismatch')
    source=out/'body-force.csv';source_receipt=write_source(source,output_byte_cap=SOURCE_CAP);runs=[]
    for n in (6,9,12):
        level=out/f'n{n}'
        launches.append({'role':'native','n':n,'seconds':180})
        run_bounded([str(a.binary.resolve()),'--physical',str(n),str(source),str(level)],
                    deadline=deadline,seconds=180,log_path=out/f'n{n}.log')
        require({x.name for x in level.iterdir()}=={'records.jsonl','acquisition.txt','owned-state.bin'},'unexpected native output')
        require(sum(x.stat().st_size for x in level.iterdir())<=NATIVE_CAP,'native shared writer cap')
        # Native process exited before the independent reference is consulted.
        comparison_file=out/f'n{n}-comparison.json';launches.append({'role':'comparison','n':n,'seconds':180})
        run_bounded([sys.executable,'-B',str(repo/'tools/flat_wall_compare_worker.py'),'--physical-records',
                     '--records',str(level/'records.jsonl'),'--source',str(source),'--head',head,
                     '--binary',str(a.binary.resolve()),'--output',str(comparison_file),
                     '--output-byte-cap',str(COMPARISON_CAP)],
                    deadline=deadline,seconds=180,log_path=out/f'n{n}-comparison.log')
        comparison=json.loads(comparison_file.read_text())
        reference_force=[-1.,0.,0.];reference_torque=[0.,-1./60.,-.5];measurements=[]
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
        require(file_digest(a.binary)==a.binary_sha256,'binary changed during campaign')
        require(git('rev-parse','HEAD')==head and not git('status','--porcelain=v1','--ignore-submodules=all'),'source changed during campaign')
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
        'native_managed_cap':10000000,'comparison_managed_cap':MAX_BYTES,
        'aggregate_deadline_seconds':TOTAL_SECONDS,'work_deadline_seconds':TOTAL_SECONDS-CLEANUP_SECONDS,
        'sampled_rss_threshold_bytes':RSS_THRESHOLD,'rss_sample_seconds':SAMPLE_SECONDS,'continuous_rss_bound':False,
        'output_cap_bytes':OUTPUT_CAP,'output_reserved_quotas':QUOTAS,
        'prescribed_launches':launches,'prescribed_process_count_including_supervisor_controller':2+len(launches),
        'launch_count_is_not_universal_os_process_creation_bound':True}
    write_reserved(out/'campaign-report.json',(json.dumps(report,indent=2)+'\n').encode(),SUMMARY_CAP)
    # Accounting consistency check supplements the guards; it never substitutes
    # for bounded producer writes. Receipt/controller-log slots are reserved.
    require(sum(x.stat().st_size for x in out.rglob('*') if x.is_file())<=OUTPUT_CAP,'output accounting mismatch')

def main():
    if sys.argv[1:2]==['--guard-controller']:
        controller(sys.argv[2:]);return 0
    receipt=supervise(None,None,started=ENTRY_TIME,prepare=lambda:prepare(sys.argv[1:]))
    return 0 if receipt['status']=='completed' else 1
if __name__=='__main__':sys.exit(main())
