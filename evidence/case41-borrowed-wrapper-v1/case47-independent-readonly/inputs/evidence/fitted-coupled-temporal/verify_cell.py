"""Replay all actual quadratic-cell equations, including its work rejection."""
import argparse
import copy
import json
import numpy as np
import model as m
from cell import initial,quadrature


def close(actual,expected,tolerance=1e-11):
    a,b=np.asarray(actual,float),np.asarray(expected,float)
    m.require(a.shape==b.shape and np.all(np.isfinite(a))and np.max(np.abs(a-b),initial=0)<=tolerance,'complete actual cell arithmetic')


def verify(data):
    unknown=np.array(data['unknowns'],float)
    m.require(unknown.shape==(22,) and np.all(np.isfinite(unknown)),'complete six-acceleration / 16-pressure system')
    q0,eta0=initial(data['initial_kind']);h=data['interval']
    m.require(0<h<=.05,'bounded qualified cell interval')
    actual=quadrature(q0,eta0,unknown,h,32,True)
    close(data['integrated_momentum_residual'],actual['residual'])
    close(data['integrated_convection'],actual['convection_integral'])
    close(data['actual_local_GCL_defect'],actual['gcl'])
    m.require(np.linalg.norm(actual['residual'])<=1e-11,'all 22 actual integrated momentum equations')
    close(data['max_integrated_momentum_residual'],np.max(np.abs(actual['residual'])))
    close(data['integrated_momentum_norm'],np.linalg.norm(actual['residual']))
    for name in ['max_strong_divergence','max_strong_acceleration','max_material_mismatch']:
        close(data[name],actual[name]);m.require(actual[name]<=1e-11,'full physical path constraints')
    allowance=actual['energy_allowance']
    for name,key in [('advection_loss',0),('strain_loss',1),('pressure_work',2),('actual_momentum_residual_work',3),('GCL_defect_work',4)]:
        close(data[name],actual['integrals'][key],allowance)
    for name,key in [('energy_old','energy0'),('energy_new','energy1'),('energy_ledger_error','energy_ledger_error'),('fixed_work_allowance','energy_allowance')]:
        close(data[name],actual[key],allowance)
    close(data['max_pointwise_momentum_residual'],actual['max_pointwise_momentum_residual'])
    close(data['max_pointwise_momentum_norm'],actual['max_pointwise_momentum_norm'])
    m.require(data['pointwise_momentum_gate_pass']==(actual['max_pointwise_momentum_norm']<=1e-11),'honest pointwise momentum gate')
    work_pass=abs(actual['integrals'][3])<=allowance
    m.require(not work_pass and data['work_gate_pass'] is False and data['candidate_accepted'] is False,'fixed work gate rejects actual candidate')
    m.require(data['new_public_step_enabled'] is False,'no unqualified public step')
    m.require(len(data['positive_negative_face_integrals'])==len(m.PAIR),'complete shared physical face inventory')
    for row,(i,j),positive,negative in zip(data['positive_negative_face_integrals'],m.PAIR,actual['positive'],actual['negative']):
        close(row,[i,j,positive,negative])
    m.require(len(data['samples'])==5 and len(data['quadrature_points'])==32,'complete physical trajectory replay')
    for row,time in zip(data['samples'],np.linspace(0,h,5)):
        point=m.cell_point(q0,eta0,unknown[:6],time,unknown[6:]);s=point['spatial']
        close(row['time'],time,0.)
        for name,exact in [('q',point['q']),('eta',point['eta']),('nodes',s['pos']),('velocity',point['flux']['velocity']),
                           ('reduced_velocity',point['z']),('mass',s['mass']),('reconstructed_pressure',s['Q']@unknown[6:]),
                           ('divergence',s['D']@point['z']),('pointwise_momentum_residual',point['momentum_residual'])]:close(row[name],exact)
    for recorded,replayed in zip(data['quadrature_points'],actual['points']):
        m.require(set(recorded)==set(replayed),'complete actual quadrature point')
        for name in recorded:close(recorded[name],replayed[name])
    return dict(interval=h,integrated_momentum_residual=float(np.max(np.abs(actual['residual']))),
                local_GCL_defect=float(np.max(np.abs(actual['gcl']))),residual_work=float(actual['integrals'][3]),
                fixed_work_allowance=allowance,work_gate_pass=False,accepted_advancing_steps=0)


def endpoint_convection(data):
    u=np.array(data['samples'][-1]['velocity']);convection=np.zeros((16,3))
    for i,j,positive,negative in data['positive_negative_face_integrals']:
        route=positive*u[i]-negative*u[j]
        convection[i]+=route;convection[j]-=route
    return np.einsum('ndj,nd->j',m.R_NODE,convection)


def negative_tests(data):
    for kind in ['false_acceptance','zero_pressure','omit_normal_acceleration','zero_work_defect',
                 'hide_pointwise_momentum','endpoint_momentum_flux','face_circulation','stale_endpoint_mass']:
        bad=copy.deepcopy(data)
        if kind=='false_acceptance':bad['candidate_accepted']=True;bad['work_gate_pass']=True
        elif kind=='zero_pressure':bad['unknowns'][6:]=[0.]*16
        elif kind=='omit_normal_acceleration':bad['unknowns'][2]=0.
        elif kind=='zero_work_defect':bad['actual_momentum_residual_work']=0.
        elif kind=='hide_pointwise_momentum':bad['max_pointwise_momentum_residual']=0.;bad['pointwise_momentum_gate_pass']=True
        elif kind=='endpoint_momentum_flux':bad['integrated_convection']=endpoint_convection(data).tolist()
        elif kind=='face_circulation':
            rows=bad['positive_negative_face_integrals']
            before=np.zeros(16)
            for i,j,p,n in rows:before[i]+=p-n;before[j]-=p-n
            lookup={(i,j):k for k,(i,j,p,n)in enumerate(rows)}
            tri=list(map(int,m.IDS[m.TRI[0]]))
            for i,j in zip(tri,tri[1:]+tri[:1]):rows[lookup[min(i,j),max(i,j)]][2 if i<j else 3]+=1/1024
            after=np.zeros(16)
            for i,j,p,n in rows:after[i]+=p-n;after[j]-=p-n
            m.require(np.max(np.abs(before-after))<1e-15,'zero-balance circulation control')
        else:bad['samples'][-1]['mass']=bad['samples'][0]['mass'][:]
        try:verify(bad)
        except ValueError:print('REJECTED',kind)
        else:raise ValueError('cell corruption accepted: '+kind)


if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('payload');parser.add_argument('--negative-self-test',action='store_true');args=parser.parse_args()
    data=json.load(open(args.payload));result=verify(data)
    if args.negative_self_test:negative_tests(data)
    print('PASS',json.dumps(result,sort_keys=True))
