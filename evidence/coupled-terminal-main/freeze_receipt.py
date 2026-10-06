"""Freeze completed qualification without overwriting an earlier receipt."""
from pathlib import Path
import datetime,hashlib,json,re,subprocess,sys
E=Path(__file__).resolve().parent;ROOT=E.parents[1]
source,run,jobs_file=sys.argv[1:]
if (E/'receipt.json').exists():raise ValueError('refuse overwrite receipt')
jobs=json.loads((E/jobs_file).read_text())['response']['structuredContent']['jobs']
logs={job['name']:'hosted-repair-'+('gate' if job['name']=='core-and-executable' else 'executable' if job['name']=='executable-contracts' else job['name'].split('(')[1][:-1])+'.log' for job in jobs}
spans={}
for name,path in logs.items():
 times=re.findall(r'(?m)^(2026-\S+Z) ',(E/path).read_text())
 dt=lambda x:datetime.datetime.fromisoformat(x.replace('Z','+00:00'))
 spans[name]=(dt(times[-1])-dt(times[0])).total_seconds()
paths=subprocess.check_output(['git','ls-tree','-r','--name-only',source,'src','tests','examples','Cargo.toml','Cargo.lock','rust-toolchain.toml','.github/workflows/rust-rheon.yml','tools/test_column_interface_verifier.py'],cwd=ROOT,text=True).splitlines()
source_hashes={p:hashlib.sha256(subprocess.check_output(['git','show',source+':'+p],cwd=ROOT)).hexdigest()for p in paths}
evidence_hashes={str(p.relative_to(E)):hashlib.sha256(p.read_bytes()).hexdigest()for p in sorted(E.rglob('*'))if p.is_file()and '__pycache__'not in p.parts and p.name!='receipt.json'}
r={'base':'773bd2725e35590cfe9cbca5625f5239b98f03c2','source':source,'source_tree':subprocess.check_output(['git','rev-parse',source+'^{tree}'],cwd=ROOT,text=True).strip(),'local_source':'c0591ebc4c2619d5ad6e6decfe9d3957ace8db5b','source_sha256':source_hashes,'evidence_sha256':evidence_hashes,'hosted_run':int(run),'hosted_run_url':'https://github.com/MrScripty/Rheon/actions/runs/'+run,'hosted_jobs_file':jobs_file,'hosted_job_logs':logs,'hosted_log_span_seconds':spans,'physics_claims_changed':False,'frozen_research_heads':{'forcing':'f76841e9217e781d91c5bfeabadb6b4c6f42b20b','diagnosis':'dad53b4054034fe0c2ff6240464df7441ab4e6a9'}}
(E/'receipt.json').write_text(json.dumps(r,indent=2)+'\n')
print('Frozen receipt SHA256 '+hashlib.sha256((E/'receipt.json').read_bytes()).hexdigest())
