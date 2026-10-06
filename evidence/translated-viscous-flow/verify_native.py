"""Independently replay each native accepted step and finite geometry trajectory."""
import argparse,copy,json
from pathlib import Path
import numpy as np
from reference import assembly,exact_semidiscrete,require

def close(x,y,label,tol=2e-11):
    require(np.all(np.isfinite(np.asarray(x,float))) and np.allclose(x,y,rtol=0,atol=tol),label)

def verify(data):
    a=assembly();dt=data['dt'];steps=data['states'];mass=a['mass'];M=a['M'];K=a['K'];R=a['R'];z=a['z0'].copy();u=np.column_stack((np.full(len(mass),.25),np.zeros(len(mass)),R@z));time=0.;offset=0.;work=[];drifts=[];total_drifts=[]
    require(data['allocated_bytes']==59184,'bounded native payload')
    close(data['mass'],mass,'native independent masses',2e-15)
    close(np.asarray(data['nodes'])[:,:2],a['points'],'corrected periodic physical nodes',2e-15)
    require(np.array_equal(np.asarray(data['nodes'])[:,2],a['ids']),'periodic identity')
    require(data['triangles']==[list(t) for t in a['triangles']],'micro topology')
    for i,entry in enumerate(steps):
        s=entry if i==0 else entry['state']
        if i:
            z=np.linalg.solve(M+dt*K,M@z);u[:,2]=R@z;time+=dt;offset+=.25*dt
        require(s['version']==i,'accepted common version')
        require(s['pressure']==0.,'analytic atmospheric relative pressure')
        close(s['time'],time,'accepted physical clock',2e-15);close(s['offset'],offset,'velocity-derived cap motion',2e-15)
        close(s['physical_nodes'],a['points']+np.array([offset,0.]),'finite physical material geometry',2e-15)
        close(s['velocity'],u,'independent dense BE solution')
        close(s['liquid_volume'],mass/3.,'accepted nodal liquid volume',2e-15)
        require(s['liquid_volume']==steps[0]['liquid_volume'],'bitwise accepted nodal volume conservation')
        if i:
            r=entry['report'];v=np.array(s['velocity'])
            old=steps[i-1] if i==1 else steps[i-1]['state'];oldv=np.array(old['velocity'])
            e0=.5*float(np.sum(mass[:,None]*oldv**2));e1=.5*float(np.sum(mass[:,None]*v**2));inc=.5*float(np.sum(mass[:,None]*(v-oldv)**2));zz=v[a['free'],2];oldz=oldv[a['free'],2];res=(M+dt*K)@zz-M@oldz;power=float(zz@K@zz);rw=float(zz@res)
            require(r['mass']==data['states'][1]['report']['mass'],'bitwise accepted mass conservation')
            close(r['mass'],sum(mass),'liquid mass',2e-15)
            close(r['momentum_before'],np.sum(mass[:,None]*oldv,axis=0),'old full momentum')
            close(r['momentum_after'],np.sum(mass[:,None]*v,axis=0),'new full momentum')
            close(r['momentum_after'],r['momentum_before'],'momentum conservation')
            initial_p=np.sum(mass[:,None]*np.array(steps[0]['velocity']),axis=0)
            total_drifts.append(float(np.max(np.abs(np.array(r['momentum_after'])-initial_p))))
            close(r['momentum_after'],initial_p,'whole-trajectory momentum conservation')
            close([r['energy_before'],r['energy_after'],r['increment_energy'],r['strain_power']],[e0,e1,inc,power],'physical BE work terms')
            close(r['residual'],np.linalg.norm(res),'true residual',2e-14)
            close(r['residual_work'],rw,'true residual work',2e-14)
            close(r['work_error'],e1-e0+inc+dt*power-rw,'full BE work identity',3e-14)
            require(e1<e0 and power>0 and r['iterations']>0,'actual viscous evolution')
            require(r['residual']<1e-11 and abs(r['work_error'])<1e-12 and r['divergence']<1e-12,'native acceptance gates')
            work.append(abs(e1-e0+inc+dt*power-rw));drifts.append(float(np.max(np.abs(np.array(r['momentum_after'])-r['momentum_before']))))
    require(data['rollback_trials']==4*(len(steps)-1),'all native rollback stages every step')
    last=np.array(steps[-1]['state']['velocity'])[a['free'],2];d=last-exact_semidiscrete(a,time)
    return {'dt':dt,'steps':len(steps)-1,'time':time,'offset':offset,'temporal_L2_error':float(np.sqrt(d@M@d)),'max_work_error':max(work),'max_momentum_drift':max(drifts),'total_momentum_drift':max(total_drifts),'reference':'evidence/fitted-height-periodic-repair/reference.py spatial values; new co-moving trajectory and dense BE; no old fitted-height-formulation path','scope':'restricted invariant flow; semidiscrete temporal accuracy only'}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('input');p.add_argument('--negative-self-test',action='store_true');args=p.parse_args();data=json.loads(Path(args.input).read_text());result=verify(data)
    if args.negative_self_test:
        for key in ['liquid_volume','velocity','offset','time','pressure','physical_nodes','mass','energy_after']:
            bad=copy.deepcopy(data);entry=bad['states'][2];s=entry['state']
            if key=='liquid_volume':s[key][0]+=.01
            elif key=='velocity':s[key][0][2]+=.01
            elif key=='physical_nodes':s[key][0][0]+=.01
            elif key in ['mass','energy_after']:entry['report'][key]+=.01
            else:s[key]+=.01
            try:verify(bad)
            except ValueError:print('REJECTED',key)
            else:raise ValueError('corrupted evidence accepted: '+key)
    print(json.dumps(result,indent=2))
