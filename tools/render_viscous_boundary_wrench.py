"""Render actual fixed native/continuum controls; derive no renderer physics."""
import argparse
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import subprocess
from check_viscous_boundary_wrench import read_json, scalar, verify
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'research'))
from check_proof_audit import require_external_output

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def render(coarse,bounded,output):
    output=require_external_output(output)
    output.parent.mkdir(parents=True,exist_ok=True)
    controls=[]
    name='polynomial-tilted-curl'
    for path,case in [(coarse,'polynomial-coarse'),(bounded,'polynomial-bounded')]:
        verified=verify(path,case)
        raw=read_json(path)
        field=next(f for f in raw['fields'] if f['name']==name)
        physical=verified['polynomial_physical_controls'][name]['continuum_wrench_exact']
        controls.append({'case':case,'raw_path':str(path),'raw_sha256':sha(path),
                         'observed_native_wrench':list(map(str,map(scalar,field['solid_wrench']))),
                         'continuum_wrench':physical,'verified_receipt':verified})
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    fig,axes=plt.subplots(1,2,figsize=(10,4.4),layout='constrained')
    for axis,index,label in [(axes[0],1,'Solid force y (N)'),(axes[1],5,'Solid torque z (N m)')]:
        observed=[float(Fraction(c['observed_native_wrench'][index])) for c in controls]
        physical=[float(Fraction(c['continuum_wrench'][index])) for c in controls]
        axis.bar([-.18,.82],observed,width=.36,color='#285f85',label='Actual native diagnostic')
        axis.bar([.18,1.18],physical,width=.36,color='#bb5b3f',label='Exact continuum integral')
        axis.axhline(0,color='#444',linewidth=.7)
        axis.set_xticks([0,1],['One-cell box','Two-cell box'])
        axis.set_ylabel(label)
        axis.grid(axis='y',alpha=.2)
    axes[0].legend(fontsize=8)
    fig.suptitle('Fixed solid-local stress controls: continuum traction accuracy fails',fontsize=12)
    fig.text(.5,.005,'Stationary variational diagnostic only; no pressure solve, advancing coupling or refinement claim.',ha='center',fontsize=8)
    output.mkdir()
    image=output/'viscous-wrench-physical-limit.png';fig.savefig(image,dpi=180);plt.close(fig)
    receipt={'kind':'render_of_verified_fixed_native_controls','source_head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=Path(__file__).resolve().parents[1],text=True).strip(),
             'renderer_sha256':sha(Path(__file__)),'oracle_source_sha256':sha(Path(__file__).with_name('check_viscous_boundary_wrench.py')),
             'controls':controls,'image_sha256':sha(image),'renderer_derives_physics':False,
             'continuum_accuracy_qualified':False,'refinement_campaign':False}
    (output/'render-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
    return {'image':str(image),'image_sha256':sha(image)}

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('coarse',type=Path);p.add_argument('bounded',type=Path);p.add_argument('output',type=Path)
    args=p.parse_args();print(json.dumps(render(args.coarse,args.bounded,args.output)))
