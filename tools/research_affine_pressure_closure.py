"""Exact research-only paired-wall affine pressure trace; not a native solver.

This investigates one alternative using fluid centers on OPPOSITE sides of the
box. Its H^T changes virtual wall-flux allocation. No production pressure map,
metric, unknown or stepping policy is selected. Run with a NEW external path.
"""
from fractions import Fraction as F
import argparse
import json
from pathlib import Path
import subprocess

def paired_trace(a,b,cl,ch,offset,gradient,velocity):
    if not cl<a<b<ch:
        raise ValueError('strict opposite-side center support required')
    hlow=((ch-a)/(ch-cl),(a-cl)/(ch-cl))
    hhigh=((ch-b)/(ch-cl),(b-cl)/(ch-cl))
    p=(offset+gradient*cl,offset+gradient*ch)
    low=sum(h*x for h,x in zip(hlow,p))
    high=sum(h*x for h,x in zip(hhigh,p))
    force=low-high  # unit area, solid left normal negative / right positive
    constraint=tuple(velocity*(l-h) for l,h in zip(hlow,hhigh))
    work=sum(x*y for x,y in zip(p,constraint))
    if low!=offset+gradient*a or high!=offset+gradient*b:
        raise ValueError('affine reproduction failed')
    if force!=-gradient*(b-a) or work!=velocity*force:
        raise ValueError('force/transpose pairing failed')
    if sum(hlow)!=1 or sum(hhigh)!=1 or sum(constraint)!=0:
        raise ValueError('constant/gauge closure failed')
    return {'walls':list(map(str,(a,b))), 'opposite_fluid_centers':list(map(str,(cl,ch))),
            'H_low':list(map(str,hlow)), 'H_high':list(map(str,hhigh)),
            'wall_pressure':list(map(str,(low,high))), 'force_unit_area':str(force),
            'matched_virtual_cell_flux':list(map(str,constraint)), 'joint_work':str(work),
            'physical_adjacent_cell_flux':list(map(str,(velocity,-velocity))),
            'same_local_geometric_flux':constraint==(velocity,-velocity)}

def research():
    fixtures=[(F(1),F(2),F(1,2),F(5,2),F(7),F(15,4),F(2)),
              (F(-2),F(3),F(-9,4),F(15,4),F(-5),F(-7,3),F(1,2)),
              (F(10),F(11),F(39,4),F(45,4),F(0),F(3),F(-2))]
    return {'kind':'exact_rational_pressure_closure_research_only',
            'native_pressure_implementation':False, 'closure_selected':False,
            'new_solver_selected':False, 'advancing_coupling_authorized':False,
            'alternative':'paired_opposite_wall_affine_trace_with_its_transpose',
            'cases':[paired_trace(*f) for f in fixtures],
            'open_obligations':['three-dimensional patch/moment quadrature and stored arithmetic',
                                'virtual versus geometric cell flux and kinetic metric choice',
                                'physical accuracy beyond affine pressure, source compatibility and conditioning',
                                'independent owner design review before any pressure implementation']}

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('output',type=Path)
    output=parser.parse_args().output.resolve()
    if output.exists(): raise ValueError('new output required')
    check=subprocess.run(['git','-C',str(output.parent),'rev-parse','--is-inside-work-tree'],capture_output=True)
    if check.returncode==0: raise ValueError('output must be outside Git')
    with output.open('x') as stream: json.dump(research(),stream,indent=2);stream.write('\n')
