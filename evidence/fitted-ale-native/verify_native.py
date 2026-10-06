"""Independent native finite ALE replay: actual face integrals precede marginals."""
import argparse,copy,json,sys
from pathlib import Path
import numpy as np
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'evidence/fitted-finite-transport'))
from reference import ref,symbolic_path,require
from numerical import actual,finite,donor,initial,ode_reference,limitations

def close(a,b,label,tol=2e-11):
    require(np.all(np.isfinite(np.asarray(a,float)))and np.allclose(a,b,rtol=0,atol=tol),label)

def face_dict(entry):
    result={}
    for row in entry['transfers']:
        pair=tuple(map(int,row[:2]));require(pair[0]<pair[1]and pair not in result,'one shared oriented face identity');require(all(np.isfinite(row))and all(x>=0 for x in row[2:]),'nonnegative separated integrated transfers');result[pair]=row
    return result

def verify(data):
    require(data['allocated_bytes']==136352,'charged bounded owner/workspace payload')
    spatial=symbolic_path();R=spatial['R'];ids=spatial['ids'];z=initial();u=np.column_stack((np.full(32,.25),np.zeros(32),R@z));time=0.;offset=0.;m0,_,_,_,_,p0=actual(0.);initial_p=np.sum(m0[:,None]*u,axis=0);allwork=[];alldrift=[];allgcl=[];allquadrature=[];allface=[];allres=[]
    close(np.array(data['nodes'])[:,:2],p0,'corrected initial geometry',2e-15);require(np.array_equal(np.array(data['nodes'])[:,2],ids),'periodic topology identities');require(data['triangles']==[list(t)for t in spatial['micro']],'microtriangle topology')
    for index,entry in enumerate(data['states']):
        s=entry if index==0 else entry['state'];dt=0. if index==0 else entry['report']['dt'];old_u=u.copy();old_m=actual(offset)[0];previous_offset=offset
        if index:
            require(dt>0 and dt<=data['dt'],'actual requested final interval, no fabricated fixed dt');time+=dt;offset+=.25*dt
            close(s['offset'],offset,'cap displacement driven by accepted velocity',2e-15)
            # Use the actually published interval endpoint as independent reference.
            offset=s['offset'];require(0<=offset<=.125,'strict qualified cap path')
        m,_,K,_,_,points=actual(offset)
        if index:
            faces=face_dict(entry);F=finite(ref.Q(previous_offset),ref.Q(offset));require(set(F)<=set(faces),'complete physical nonzero transfer inventory')
            err=0.
            for pair,row in faces.items():
                f=F.get(pair,0.);close([row[2],row[3]],[max(f,0.),max(-f,0.)],'actual integrated physical face: '+str(pair),3e-15);err=max(err,abs(row[2]-row[3]-f))
            allface.append(err);A=donor(m,F);operator=R.T@(A+dt*K)@R;rhs=R.T@(old_m*old_u[:,2]);z=np.linalg.solve(operator,rhs);u[:,2]=R@z
        require(s['version']==index,'one accepted version');require(s['pressure']==0.,'analytic pressure, no unqualified pressure solve');close(s['time'],time,'accepted physical time',2e-15);close(s['physical_nodes'],points,'actual bottom-fixed finite geometry',4e-15);close(s['velocity'],u,'independent constrained donor/viscosity solution');close(s['mass'],m,'actual geometry-derived changing nodal masses',4e-15);close(s['liquid_volume'],m/3.,'actual accepted nodal liquid volumes',2e-15)
        if index:
            r=entry['report'];v=np.array(s['velocity']);old_state=data['states'][index-1]if index==1 else data['states'][index-1]['state'];old_v=np.array(old_state['velocity']);m_native=np.array(s['mass']);old_native=np.array(old_state['mass']);free=[int(np.argmax(R[:,j]))for j in range(24)];zz=v[free,2];oldz=old_v[free,2];g=m_native-old_native;P=0.;N=0.
            An=np.diag(m_native)
            for (i,j),row in faces.items():
                p,n=row[2:4];g[i]+=p-n;g[j]-=p-n;An[i,i]+=p;An[j,i]-=p;An[j,j]+=n;An[i,j]-=n
            rz=R.T@(An+dt*K)@R@zz-R.T@(old_native*old_v[:,2]);rx=R.T@(.25*g);norm=float(np.sqrt(rz@rz+rx@rx));rw=float(zz@rz+.25*sum(rx));e0=.5*float(np.sum(old_native[:,None]*old_v**2));e1=.5*float(np.sum(m_native[:,None]*v**2));inc=.5*float(np.sum(old_native[:,None]*(v-old_v)**2));adv=sum((row[2]+row[3])*float(np.sum((v[i]-v[j])**2))/2 for (i,j),row in faces.items());strain=float(zz@(R.T@K@R)@zz);gw=.5*float(g@np.sum(v*v,axis=1));work=e1-e0+inc+adv+dt*strain-rw+gw
            close([r['mass_before'],r['mass_after']],[sum(old_native),sum(m_native)],'liquid mass totals',2e-15);close(r['mass_after'],3.5625,'exact total liquid mass',2e-15);close(r['momentum_before'],np.sum(old_native[:,None]*old_v,axis=0),'old full momentum');close(r['momentum_after'],np.sum(m_native[:,None]*v,axis=0),'new full momentum');close(r['momentum_after'],initial_p,'whole trajectory full momentum');close([r['energy_before'],r['energy_after'],r['increment_loss'],r['advection_loss'],r['strain_power']],[e0,e1,inc,adv,strain],'full changing-mass transport/strain ledger',3e-14);close(r['residual'],norm,'true full composed momentum residual',2e-14);close(r['residual_work'],rw,'true full residual work',3e-14);close(r['gcl_work'],gw,'measured geometric defect work sign',2e-16);close(r['work_error'],work,'changing-mass work identity',3e-14);close(r['gcl_max'],max(abs(g)),'actual finite GCL',2e-16)
            quad=max(max(abs(row[2]-row[4]),abs(row[3]-row[5]))for row in faces.values());close(r['quadrature_error'],quad,'physical quadrature refinement');close(r['relative_transfer_l1'],sum(row[2]+row[3]for row in faces.values()),'nonzero actual relative transfers');require(r['relative_transfer_l1']>0 and adv>0 and strain>0 and e1<e0,'genuine relative transport plus viscosity');require(r['iterations']>0 and r['residual']<1e-11 and r['divergence']<1e-12 and abs(r['work_error'])<1e-12,'actual acceptance gates')
            allwork.append(abs(work));alldrift.append(float(np.max(np.abs(np.array(r['momentum_after'])-initial_p))));allgcl.append(max(abs(g)));allquadrature.append(quad);allres.append(norm)
    require(data['rollback_trials']==6,'all six native cancellation stages after nonzero accepted state')
    exact=ode_reference()[0];last=data['states'][-1]['state'];last_z=np.array(last['velocity'])[free,2];d=R@(last_z-exact);error=float(np.sqrt(d@(m*d)))
    # Endpoint temporal study is at physical time .5, with the actual final dt
    # explicitly serialized. Any larger change of the physical interval fails.
    close(time,.5,'actual final physical time',2e-15)
    return {'dt':data['dt'],'steps':len(data['states'])-1,'actual_final_dt':data['states'][-1]['report']['dt'],'time':time,'cap_offset':offset,'temporal_L2_error':error,'max_total_momentum_drift':max(alldrift),'max_work_error':max(allwork),'max_GCL':max(allgcl),'max_quadrature_refinement':max(allquadrature),'max_physical_face_integral_error':max(allface),'max_true_residual':max(allres),'scope':'native nonzero relative transport and viscosity in preceding invariant flow; no general pressure or continuum spatial accuracy claim'}

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('input');p.add_argument('--negative-self-test',action='store_true');args=p.parse_args();data=json.loads(Path(args.input).read_text());result=verify(data)
    if args.negative_self_test:
        for variant in ['velocity','cap_offset','clock','pressure','physical_geometry','liquid_volume','energy','face_circulation','endpoint_frozen_flux']:
            bad=copy.deepcopy(data);e=bad['states'][2];s=e['state']
            if variant=='velocity':s['velocity'][0][2]+=.01
            elif variant=='cap_offset':s['offset']+=.01
            elif variant=='clock':s['time']+=.01
            elif variant=='pressure':s['pressure']=.01
            elif variant=='physical_geometry':s['physical_nodes'][0][0]+=.01
            elif variant=='liquid_volume':s['liquid_volume'][0]+=.01
            elif variant=='energy':e['report']['energy_after']+=.01
            elif variant=='face_circulation':
                rows={tuple(map(int,r[:2])):r for r in e['transfers']};a,b,c=[int(data['nodes'][i][2])for i in data['triangles'][0]];balance=np.zeros(32)
                for i,j in [(a,b),(b,c),(c,a)]:
                    pair=(min(i,j),max(i,j));row=rows[pair]
                    if i<j:row[2]+=.001
                    else:row[3]+=.001
                    balance[i]+=.001;balance[j]-=.001
                require(max(abs(balance))==0.,'real negative circulation has zero nodal balance')
            else:
                f=actual(s['offset'])[3];dt=e['report']['dt']
                for row in e['transfers']:
                    qf=dt*f.get(tuple(map(int,row[:2])),0.);row[2]=max(qf,0.);row[3]=max(-qf,0.)
            try:verify(bad)
            except ValueError:print('REJECTED',variant)
            else:raise ValueError('corrupted native evidence accepted: '+variant)
    print(json.dumps(result,indent=2))
