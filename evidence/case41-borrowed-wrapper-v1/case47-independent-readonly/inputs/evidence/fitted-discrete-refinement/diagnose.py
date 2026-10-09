"""Actual fine-step trace: each accepted cell and failed Newton history saved."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys
import numpy as np
import temporal as repaired
ROOT=Path(__file__).resolve().parents[2]

def original_traced():
 source=(ROOT/'evidence/fitted-discrete-work/step.py').read_text()
 source=source.replace("def solve(h,kind='initial',state=None,diagnostics=True):","def solve(h,kind='initial',state=None,diagnostics=True,trace=None):").replace("residual=equation(unknown)\n  if","residual=equation(unknown)\n  if trace is not None:trace.append(dict(iteration=iteration+1,rate_norm=float(np.linalg.norm(residual)),unknowns=unknown.tolist(),equation_calls=calls))\n  if")
 ns={'__file__':str(ROOT/'evidence/fitted-discrete-work/step.py'),'__name__':'traced_original'};exec(compile(source,'<frozen-step-with-observation-only>','exec'),ns)
 return ns

def run(mode,out):
 api=vars(repaired)if mode=='stable'else original_traced();q,eta=api['initial']();rows=[]
 for n in range(16):
  trace=[];data=dict(mode=mode,interval=.00625,step=n+1,initial_q=q.tolist(),initial_eta=eta.tolist(),trace=trace,completed_steps=rows)
  try:c=api['solve'](.00625,state=(q,eta),diagnostics=False,trace=trace)
  except ValueError as error:
   data.update(outcome='REJECTED',exception=str(error));out.write_text(json.dumps(data,indent=2)+'\n');print(mode,'REJECTED step',n+1,'norms',[r['rate_norm']for r in trace],flush=True);return
  rows.append(c);q=np.array(c['end_q']);eta=np.array(c['end_eta']);data.update(outcome='in progress');out.write_text(json.dumps(data,indent=2)+'\n');print(mode,'completed',n+1,'rate',c['finite_momentum_rate_norm'],flush=True)
 data.update(outcome='PASS',completed_steps=rows,final_q=q.tolist(),final_eta=eta.tolist());out.write_text(json.dumps(data,indent=2)+'\n')

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('mode',choices=['original','stable']);p.add_argument('output');args=p.parse_args();run(args.mode,Path(args.output))
