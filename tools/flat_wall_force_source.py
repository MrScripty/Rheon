#!/usr/bin/env python3
"""Generate ONLY body-force density coefficients. No velocity/load table or solve.

Local support coordinates (s,y,t)=(x-1,y,z-1). Polynomial factor derivation is
recorded here for independent re-derivation. Native provider consumes the emitted
coefficients, not these factors, an analytic field or a reference wrench.
"""
import argparse, hashlib, json
from fractions import Fraction as F
from math import comb
from pathlib import Path

def derivative(p,n):
    for _ in range(n): p={i-1:i*c for i,c in p.items() if i}
    return p
def multiply(p,q):
    r={}
    for i,c in p.items():
        for j,d in q.items(): r[i+j]=r.get(i+j,F(0))+c*d
    return {i:c for i,c in r.items() if c}
def coefficients():
    x={6+i:F((-1)**i*comb(6,i)) for i in range(7)}
    z=multiply(x,{0:F(1,2),1:F(1)}); y={4:F(1),5:F(-2),6:F(1)}
    amplitude=F(12012**2,2); result={}
    # fx=-A(X''ZY'+XZ''Y'+XZY'''); fy=A(X'''ZY+X'Z''Y+X'ZY'').
    for axis,sign,orders in [(0,-1,(2,1,0)),(0,-1,(0,1,2)),(0,-1,(0,3,0)),
                             (1,1,(3,0,0)),(1,1,(1,0,2)),(1,1,(1,2,0))]:
        factors=[derivative(p,n) for p,n in zip([x,y,z],orders)]
        for i,c in factors[0].items():
            for j,d in factors[1].items():
                for k,e in factors[2].items():
                    key=(axis,i,j,k);result[key]=result.get(key,F(0))+sign*amplitude*c*d*e
    result={key:c for key,c in result.items() if c}
    assert len(result)<=1024
    for c in result.values(): assert F.from_float(float(c))==c
    return result
def write(path):
    terms=coefficients()
    text='# body-force density ONLY, SI; s=x-1,y=y,t=z-1; support[1,2]x[0,1]x[1,2]\n'
    text+=''.join(f'{a},{float(c):.17g},{i},{j},{k}\n' for (a,i,j,k),c in sorted(terms.items()))
    path=Path(path);path.parent.mkdir(parents=True,exist_ok=True)
    with path.open('x') as f:f.write(text)
    return {'terms':len(terms),'sha256':hashlib.sha256(text.encode()).hexdigest(),'physical_solve_executed':False}
if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('output');args=parser.parse_args()
    print(json.dumps(write(args.output),indent=2))
