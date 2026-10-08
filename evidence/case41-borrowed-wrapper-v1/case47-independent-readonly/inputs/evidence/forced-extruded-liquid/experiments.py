"""Pre-implementation impulse/work checks; no compiled-step claim."""
from fractions import Fraction as Q
import json
import numpy as np
import reference as r


def exact_constant_load():
    m0=[Q(1),Q(1)];m1=[Q(3,4),Q(5,4)];h=Q(1,2);w0=Q(1,2);rows=[]
    for a in [Q(1,4),Q(-1,4)]:
        w1=w0+h*a
        # A=[[1,0],[-1/4,5/4]], A1=m0, from one actual shared transfer.
        rhs=[mm*(w0+h*a)for mm in m0]
        solved=[rhs[0],(rhs[1]+rhs[0]/4)/Q(5,4)]
        r.require(solved==[w1,w1],'exact constant acceleration on changing masses')
        e0=sum(mm*w0*w0/2 for mm in m0);e1=sum(mm*w1*w1/2 for mm in m1)
        be=sum(mm*(w1-w0)**2/2 for mm in m0);work=h*sum(mm*w1*a for mm in m0)
        r.require(e1-e0+be-work==0,'exact signed coupled force work')
        wrong=[m0[i]*w0+h*m1[i]*a for i in range(2)];wrong=[wrong[0],(wrong[1]+wrong[0]/4)/Q(5,4)]
        r.require(wrong!=[w1,w1],'endpoint-mass counterexample must expose nonconstant error')
        rows.append(dict(acceleration=str(a),new_velocity=[str(w)for w in solved],energy_change=str(e1-e0),BE_loss=str(be),force_work=str(work),wrong_endpoint_mass_velocity=[str(w)for w in wrong]))
    return rows


def third_work(v,h,oldxi,a):
    m=r.m;t=r.t;R=r.third.RW
    m0=v['start']['spatial']['mass'];m1=v['end']['spatial']['mass'];old=R@oldxi
    xi=r.finite_third(v,h,old,m0,a);new=R@xi;s=v['end']['spatial'];_,kx,ky,ex,ey=r.third.blocks(s)
    plus=v['transfers']['plus'];minus=v['transfers']['minus'];c=r.third.route(plus,minus,new)
    residual=R.T@(m1*(R@(xi-oldxi))+(m1-m0)*old+c)+h*((kx+ky)@xi-R.T@(m0*a[2]))
    direct=R.T@(m1*new-m0*old+c)+h*((kx+ky)@xi-R.T@(m0*a[2]))
    e0=.5*np.dot(m0,old*old);e1=.5*np.dot(m1,new*new);be=.5*np.dot(m0,(new-old)**2)
    mix=.5*np.dot(plus+minus,(new[m.PAIR_I]-new[m.PAIR_J])**2)
    sx=h*.05*np.sum(s['area']*(ex@xi)**2);sy=h*.05*np.sum(s['area']*(ey@xi)**2)
    wg=.5*np.dot(v['gcl'],new*new);wf=h*np.dot(m0,new*a[2]);wr=np.dot(xi,residual)
    ledger=e1-e0+be+mix+sx+sy+wg-wf-wr;allow=128*t.EPS*sum(abs(x)for x in [e0,e1,be,mix,sx,sy,wg,wf,wr])
    r.require(max(np.linalg.norm(residual),np.linalg.norm(direct))/h<=r.LIMIT,'all forced third equations')
    r.require(max(abs(wr),abs(ledger))<=allow,'forced third work factor unchanged')
    impulse=np.dot(m1,new)-np.dot(m0,old)
    r.require(abs(impulse-h*3.375*a[2])<=r.LIMIT,'known third external impulse')
    return dict(coefficients=xi.tolist(),force_work=float(wf),energy_change=float(e1-e0),shear_x=float(sx),shear_y=float(sy),BE_loss=float(be),mixing_loss=float(mix),GCL_work=float(wg),residual_work=float(wr),ledger_error=float(ledger),work_allowance=float(allow),momentum_impulse=float(impulse),rate=float(np.linalg.norm(residual)/h))


def run():
    rows=[]
    for kind in ['initial','pressure_state']:
        q,eta=r.t.initial(kind)
        for sign in [1.,-1.]:
            a=sign*r.A
            for h in r.H:
                u,v,counts=r.solve(q,eta,h,a)
                nonconstant=third_work(v,h,r.third.XI,a)
                constant=third_work(v,h,np.full(12,.25),a)
                r.require(np.max(np.abs(np.array(constant['coefficients'])-(.25+h*a[2])))<=128*r.t.EPS*.25,'constant acceleration on physical changing masses')
                r.require(nonconstant['shear_x']>0 and nonconstant['shear_y']>0,'both forced third shears')
                rows.append(dict(kind=kind,acceleration=a.tolist(),h=h,unknowns=u.tolist(),planar_force_work=v['force_work'],planar_work_error=v['energy_ledger_error'],planar_work_allowance=v['fixed_work_allowance'],third_nonconstant=nonconstant,third_constant=constant,**counts))
    try:
        r.m.spatial(r.m.q_to_x(np.array([0.,.5,1.125])))
    except ValueError as error:
        flat=dict(status='REFUSED',reason=str(error),original_gate_preserved=True)
    else:
        raise ValueError('original flat-chart refusal unexpectedly disappeared')
    return dict(status='PASS',scope='Independent pre-implementation source/work experiments only; no native or flat gravity qualification',actual_first_step_cells=len(rows),exact_constant_load=exact_constant_load(),original_flat_chart=flat,rows=rows)


if __name__=='__main__':
    print(json.dumps(run(),indent=2))
