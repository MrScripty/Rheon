"""Actual negative seam-contract reproduction and exact rank sensitivity.

Old evidence remains valid for its actual periodic P1 geometry; this rejects its
claim to the intended periodic Powell–Sabin center-line seam construction.
"""
from fractions import Fraction as Q
import importlib.util
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[2]


def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path)
    module=importlib.util.module_from_spec(spec);sys.modules[name]=module;spec.loader.exec_module(module);return module


old=load('frozen_seam_reference',ROOT/'evidence/fitted-height-formulation/reference.py')
new=load('corrected_seam_reference',Path(__file__).with_name('reference.py'))


def rank(ref,points,micro,ids,constraints,bottom):
    n=max(ids)+1
    _,_,_,div,_=ref.operators(points,micro)
    rx=ref.scalar_embedding(n,constraints);ry=ref.scalar_embedding(n,constraints,bottom)
    matrix=ref.zeros(len(micro),len(rx[0])+len(ry[0]))
    for t,row in enumerate(div):
        for i in range(len(points)):
            for j,v in enumerate(rx[ids[i]]):matrix[t][j]+=row[3*i]*v
            for j,v in enumerate(ry[ids[i]]):matrix[t][len(rx[0])+j]+=row[3*i+1]*v
    return len(ref.independent_columns(matrix))


def contract(ref,points):
    p=points[11];q=(points[16][0]-1,points[16][1]);s=points[19]
    determinant=ref.cross(ref.sub(s,p),ref.sub(q,p))
    ref.require(determinant.v==0 and determinant.d==0,'periodic neighbor-center line/derivative')
    ref.require(points[30]==(s[0]+1,s[1]),'periodic seam translation')
    return {'position':[str(v.v) for v in s],'derivative':[str(v.d) for v in s]}


def diagnose():
    frozen=old.mesh();corrected=new.mesh()
    try:contract(old,frozen[0])
    except ValueError as error:negative=str(error)
    else:raise ValueError('frozen wrong seam unexpectedly passes declared geometry')
    positive=contract(new,corrected[0])
    new.require(positive['position']==['0','13/24'] and positive['derivative']==['13/96','5/192'],'independent seam values')
    old_rank=rank(old,*frozen);new_rank=rank(new,*corrected)
    new.require((old_rank,new_rank)==(33,32),'exact changed ranks')
    sensitivity=[]
    for eps in [Q(1,64),Q(1,4096),Q(1,2**30)]:
        points=list(corrected[0])
        for i in [19,30]:points[i]=(points[i][0],points[i][1]+eps)
        perturbed_rank=rank(new,points,*corrected[1:])
        new.require(perturbed_rank==33,'nonzero seam perturbation rank')
        sensitivity.append({'seam_height_perturbation':str(eps),'exact_pressure_rank':perturbed_rank,
            'scope':'geometry/pressure algebra only; no motion/GCL claim for perturbation'})
    return {'frozen_reference_source':'2554fb1c1e7270d3133b4a50b59483b3e270ac25',
        'frozen_native_source':'9afb886ed2809c61b832012c31b65904ee756f82','frozen_native_evidence':'c995671cc80f1da7826bdc15ebfc67d6d4f788aa',
        'actual_old_geometry_rejection':negative,'old_seam_position':[str(v.v) for v in frozen[0][19]],
        'old_seam_derivative':[str(v.d) for v in frozen[0][19]],'corrected':positive,
        'old_pressure_rank':old_rank,'corrected_pressure_rank':new_rank,'small_perturbations':sensitivity,
        'limits':'finite exact rank evidence does not establish uniform stability, pointwise traction or time integration'}


if __name__=='__main__':
    text=json.dumps(diagnose(),indent=2)+'\n'
    if len(sys.argv)>1:Path(sys.argv[1]).write_text(text)
    print(text,end='')
