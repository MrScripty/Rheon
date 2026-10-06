"""Plot actual independent research data; explicitly not a native replay."""
import os
os.environ.setdefault('MPLCONFIGDIR','/tmp/rheon-mpl');os.environ.setdefault('XDG_CACHE_HOME','/tmp/rheon-font-cache')
from pathlib import Path
import json,numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from numerical import actual,finite
from reference import ref
ROOT=Path(__file__).resolve().parent
data=json.loads((ROOT/'research-qualification/normal-numerical.json').read_text());F=finite(ref.Q(0),ref.Q(1,8));f=actual(.125)[3];error=np.array([F[ij]-.5*f[ij]for ij in F]);fig,axs=plt.subplots(1,3,figsize=(13,3.8),layout='constrained');axs[0].stem(range(len(error)),error,basefmt=' ');axs[0].set(xlabel='oriented shared face pair',ylabel='actual integral − frozen endpoint (kg)',title='Wrong transfers despite correct mass marginals');r=data['limitations'];axs[0].text(.02,.03,f"max face error = {r['endpoint_face_integral_error']:.6g}\nGCL still passes: {r['endpoint_frozen_flux_GCL_max']:.2g}",transform=axs[0].transAxes,fontsize=8);axs[1].bar(['old min','new min','old max','new max'],[r['old_velocity_range'][0],r['new_velocity_range'][0],r['old_velocity_range'][1],r['new_velocity_range'][1]],color=['#64748b','#b91c1c','#64748b','#2563eb']);axs[1].axhline(0,color='black',lw=.6);axs[1].set(ylabel='velocity (m/s)',title='Constrained convex bound: FAIL');rows=data['rows'];axs[2].loglog([r['dt']for r in rows],[r['temporal_L2_error']for r in rows],'o-');axs[2].grid(True,which='both',alpha=.3);axs[2].set(xlabel='dt (s)',ylabel='mass L² temporal error',title='Dense ALE reference: first-order time error');fig.suptitle('Independent finite-transport research · no native advancing claim');fig.savefig(ROOT/'finite-transport-contracts.png',dpi=150);fig.savefig(ROOT/'finite-transport-contracts.pdf');plt.close(fig)
