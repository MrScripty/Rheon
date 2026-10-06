"""Standalone scientific figure from existing-data diagnostic results."""
from pathlib import Path
import json,sys
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
P=Path(__file__).resolve().parent
r=json.loads((P/'analyze-80-normal.json').read_text())['rows'][0]
plt.rcParams.update({'svg.hashsalt':'case41-existing-data-v1','font.size':10})
fig,axes=plt.subplots(1,3,figsize=(13,4),layout='constrained')
a=axes[0];a.bar(range(22),[x*1e14 for x in r['exact_stored']['components']],color='#395b8c');a.set(xlabel='Reduced equation component',ylabel='Momentum rate × 10¹⁴',title='Existing E2 case41 / order16');a.text(.02,.98,'Components 2 + 3: 74.1% of squared norm',transform=a.transAxes,va='top',fontsize=9);a.axhline(0,color='black',linewidth=.6)
a=axes[1];keys=['chart_increment_inertia','captured_combined_force','body_force','mass_change_inertia','endpoint_donor_transport'];a.barh(['Chart inertia','Combined force','Body force','Mass change','Donor transport'],[r['term_groups'][k]['norm']for k in keys],color='#527b70');a.set_xscale('log');a.set(xlabel='Norm of exact captured term group',title='Cancellation scale')
a=axes[2];keys=['exact_stored','pressure_range','pressure_complement','rho_total_rate','native_minus_exact'];values=[r[k]['norm']for k in keys];a.barh(['Stored residual','Pressure span','Complement','Stored–latent contribution','Native–exact contribution'],values,color=['#395b8c','#6a709b','#b26a49','#527b70','#999999']);a.axvline(1e-13,color='#9f3c3c',linestyle='--',label='Original Newton target');a.set_xscale('log');a.set(xlabel='Momentum-rate norm',title='Exact diagnostic decomposition');a.legend(loc='lower right',fontsize=8)
fig.suptitle('Captured fields only • no candidate/correction • no arithmetic-floor bound',fontsize=12)
fig.savefig(sys.argv[1]if len(sys.argv)>1 else P/'case41-diagnosis.svg',metadata={'Date':None});plt.close(fig)
