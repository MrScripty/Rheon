"""Interpret a frozen static native field to expose the moving-flat obstruction.
This does not run or qualify liquid evolution. It emits no rendered simulation.
"""
import hashlib,json,math
from pathlib import Path
import subprocess
import numpy as np
ROOT=Path(__file__).resolve().parents[2]
COMMIT='a21d55dae2088aa6f9adfb2916645379c2557bcf'
FILE='evidence/column-mac/demo/pulse-y-0.25-jacobi/case.json'
data=subprocess.check_output(['git','show',COMMIT+':'+FILE],cwd=ROOT)
if data!=(ROOT/FILE).read_bytes():raise ValueError('frozen native record changed')
x=json.loads(data);step=x['steps'][0];n=x['counts'];h=x['spacing'][1];H=(2+x['top_fraction'])*h;area=x['spacing'][0]*x['spacing'][2]
normal=np.array(step['after_velocity'][1],dtype=np.float32).astype(float)
values=[float(normal[i+n[0]*(2+(n[1]+1)*z)]) for z in range(n[2]) for i in range(n[0])]
flux=math.fsum(area*u for u in values);dt=.001
if flux!=0 or not min(values)<0<max(values):raise ValueError('native kinematic witness absent')
omega=H-h;bridge=area*(omega-h/2);p1=area*omega/2
if bridge==p1:raise ValueError('normal-basis witness absent')
result={'kind':'research_obstruction_only','production_advance_executed':False,'rendered_advancing_simulation_claimed':False,'frozen_evidence_commit':COMMIT,'native_record':FILE,'native_record_sha256':hashlib.sha256(data).hexdigest(),'height_m':H,'column_area_m2':area,'surface_slot_velocity_m_per_s':values,'net_surface_flux_m3_per_s':flux,'hypothetical_kinematic_dt_s':dt,'hypothetical_euler_height_m':[H+dt*u for u in values],'hypothetical_height_spread_m':dt*(max(values)-min(values)),'normal_top_mass_bridge_kg':bridge,'normal_top_mass_P1_kg':p1,'omitted_normal_energy_j':step['export']['omitted_normal_energy'],'actual_static_physical_time_s':step['physical_time']}
if result['actual_static_physical_time_s']!=0 or result['omitted_normal_energy_j']<=0:raise ValueError('static/normal-energy scope')
Path(__file__).with_name('obstruction.json').write_text(json.dumps(result,indent=2)+'\n')
print('PASS frozen native witness: balanced nonuniform surface velocities; positive omitted normal energy; different declared normal mass models; no advancing execution')
