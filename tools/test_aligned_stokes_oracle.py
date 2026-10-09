#!/usr/bin/env python3
"""Independent exact controls and hostile-record checks; native evidence opt-in."""
import copy
from fractions import Fraction as F
import json
import math
import os
from pathlib import Path
import tempfile
import unittest

import aligned_stokes_oracle as oracle
from aligned_strain_oracle import Geometry, reference, flatten, coordinates, apply, Refusal


def unit_geometry_record(spacing=(1,1,1),origin=(0,0,0)):
    """Synthetic parser control, never presented as freshly launched native data."""
    g=Geometry((3,3,3),tuple(F.from_float(float(x)) for x in spacing),
               tuple(F.from_float(float(x)) for x in origin),(1,1,1),(2,2,2))
    faces,rows,_,_=reference(g)
    bits=oracle.hex_bits
    records={'geometry':{'counts':list(g.counts),'spacing':[bits(x) for x in g.spacing],
              'origin':[bits(x) for x in g.origin],'lower':list(g.lower),'upper':list(g.upper),
              'density':bits(g.density),'viscosity':bits(g.viscosity),
              'volumes':[bits(g.volume(p) if g.fluid(p) else 0) for p in coordinates(g.counts)]},
             'active':[],'rows':[]}
    for i,f in enumerate(faces):
        lo=list(f.p);lo[f.axis]-=1
        records['active'].append({'id':i,'axis':f.axis,'face':f.flat,'coordinates':list(f.p),
            'position':[bits(x) for x in f.position],'area':bits(f.area),'distance':bits(f.distance),
            'mass':bits(f.mass),'negative_cell':flatten(lo,g.counts),'positive_cell':flatten(f.p,g.counts)})
    for i,r in enumerate(rows):
        records['rows'].append({'id':i,'axes':list(r.family),'coordinates':list(r.p),'quadrant':r.quadrant,
            'boundary':r.closure,'weight':bits(r.weight),'terms':[[j,bits(a)] for j,a in r.terms]})
    return records


class ExactControls(unittest.TestCase):
    def test_unit_curl_exact_matched_reference(self):
        g,faces,rows,volumes=oracle.stored_geometry(unit_geometry_record())
        controls={(0,(1,0,0)):F(1,2),(0,(1,1,0)):F(-1,2),
                  (1,(0,1,0)):F(-1,2),(1,(1,1,0)):F(1,2)}
        u=[controls.get((f.axis,f.p),F(0)) for f in faces]
        self.assertEqual(oracle.energy(faces,u),F(1,2))
        self.assertTrue(all(q==0 for q in oracle.flux(g,faces,u)))
        ku,loss=apply(rows,u)
        self.assertEqual(loss,6)
        self.assertEqual(oracle.bound(rows,faces),17)
        dt=F(1,32);v=[x-dt*k/f.mass for x,k,f in zip(u,ku,faces)]
        self.assertEqual(oracle.energy(faces,v),F(679,2048))
        self.assertEqual(max(map(abs,oracle.flux(g,faces,v))),F(1,64))
        z,p,gauge=oracle.ideal_projection(g,faces,v,dt,volumes)
        self.assertEqual(oracle.energy(faces,z),F(303995,917504))
        self.assertEqual(gauge,0)
        self.assertEqual(p,[F(x,2240) for x in (0,477,280,-477,0,97,-280,-97,0,
                        0,508,266,-508,0,108,-266,-108,0,0,169,168,-169,0,69,-168,-69,0)])
        self.assertEqual(oracle.energy(faces,z)-oracle.energy(faces,v),F(-197,917504))

    def test_rounded_stability_boundary_is_not_authority(self):
        dt=math.nextafter(2.0/17.0,math.inf)
        self.assertEqual(dt*17.0,2.0)
        self.assertGreater(F.from_float(dt)*17,2)

    def test_tiny_step_cannot_claim_rounded_rest(self):
        tolerance=F.from_float(4096*math.ulp(1.0));impulse=F(1,2**200)
        with self.assertRaises(Refusal):
            oracle.certificate({'defect':[oracle.hex_bits(impulse)]*2,
                                'scale':[oracle.hex_bits(impulse)]*2,
                                'allowance':[oracle.hex_bits(tolerance*impulse)]*2},
                               impulse,impulse,tolerance,'lost update')

    def test_reflections_and_cross_component_strain_are_actual_rows(self):
        _,faces,rows,_=oracle.stored_geometry(unit_geometry_record())
        corners={}
        for row in rows:
            if row.closure==2:corners.setdefault((row.family,row.p),[]).append(row)
        self.assertEqual(len(corners),12)
        for group in corners.values():
            indices=sorted({i for r in group for i,_ in r.terms})
            self.assertEqual(len(indices),2)
            k=[[sum(r.weight*dict(r.terms).get(a,0)*dict(r.terms).get(b,0) for r in group)
                for b in indices] for a in indices]
            self.assertEqual(k[0][0],2);self.assertEqual(k[1][1],2)
            self.assertEqual(abs(k[0][1]),1);self.assertEqual(k[0][1],k[1][0])
            self.assertEqual(len(group),3)
            self.assertEqual(sorted(len(r.terms) for r in group),[1,1,2])
        self.assertEqual(sum(not r.terms for r in rows),6)

    def test_scaled_and_nonmidpoint_geometry_sign_fail_closed(self):
        for spacing,origin in [((1e-6,)*3,(0,)*3),((1e12,)*3,(0,)*3),
                                ((.3,.7,1.1),(1e8,-1e8,.1))]:
            original=unit_geometry_record(spacing,origin)
            oracle.stored_geometry(original)
            for target in ('mass','weight','coefficient'):
                bad=copy.deepcopy(original)
                if target=='mass':bad['active'][0]['mass']=oracle.hex_bits(-oracle.scalar(bad['active'][0]['mass']))
                elif target=='weight':bad['rows'][0]['weight']=oracle.hex_bits(-oracle.scalar(bad['rows'][0]['weight']))
                else:
                    row=next(r for r in bad['rows'] if r['terms'])
                    row['terms'][0][1]=oracle.hex_bits(-oracle.scalar(row['terms'][0][1]))
                with self.assertRaises(Refusal,msg=(spacing,target)):oracle.stored_geometry(bad)

    def test_pressure_gauge_flux_is_included(self):
        g,faces,_,volumes=oracle.stored_geometry(unit_geometry_record())
        u=[F(0)]*len(faces);u[0]=1
        q=oracle.flux(g,faces,u)
        self.assertNotEqual(q[0],0)
        self.assertEqual(oracle.maximum_divergence(g,faces,u,volumes),1)

    def test_pressure_metric_mismatch_is_visible(self):
        _,faces,_,_=oracle.stored_geometry(unit_geometry_record((.3,.7,1.1),(1e8,-1e8,.1)))
        # Actual mass is rounded separately from rho*area*distance. Its exact
        # stored value, not the unrounded product, defines the energy metric.
        self.assertTrue(any(f.mass!=f.area*f.distance for f in faces))
        f=next(f for f in faces if f.mass!=f.area*f.distance)
        dt=F(1,256);jump=F(1)
        legacy_update=-dt*jump/f.distance
        self.assertNotEqual(f.mass*legacy_update+dt*f.area*jump,0)

    def test_refusal_snapshots_compare_bits(self):
        r={'schema':'rheon-aligned-stokes-refusal-v1','case':'tiny-step','status':'refused','error':'Equation',
           'state_before':{'accepted_steps':1},'state_after':{'accepted_steps':1},
           'caller_before':[['0000000000000000'],[],[]],
           'caller_after':[['0000000000000000'],[],[]],
           'accepted_before':None,'accepted_after':None}
        witness={key:[oracle.hex_bits(0)] for key in ('initial','viscous','final','pressure')}
        witness['report']={key:[] for key in ('bound','step_product','viscous_energy_delta','pressure_energy_delta','initial_divergence','final_divergence','update','momentum')}
        r['accepted_before']=copy.deepcopy(witness);r['accepted_after']=copy.deepcopy(witness)
        oracle.verify_refusal(r)
        r['caller_after'][0][0]='8000000000000000'
        with self.assertRaises(Refusal):oracle.verify_refusal(r)

    def test_duplicate_json_and_invalid_interval(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'bad.json';p.write_text('{"schema":1,"schema":2}')
            with self.assertRaises(Refusal):oracle.read_json(p)
        with self.assertRaises(Refusal):oracle.interval([oracle.hex_bits(2),oracle.hex_bits(1)],'reversed')
        with self.assertRaises(Refusal):oracle.scalar(oracle.hex_bits(math.ldexp(1,-1074)))


class PrimitiveNative(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('RHEON_OUTWARD_PROBE'),'native probe not configured')
    def test_actual_primitives_and_forced_cancellation(self):
        result=oracle.qualify_probe(os.environ['RHEON_OUTWARD_PROBE'])
        self.assertTrue(result['forced_cancellation_enclosed'])
        self.assertGreater(result['primitive_cases'],350)
        self.assertGreater(result['primitive_refused'],0)

    def test_mutated_probe_result_rejected(self):
        case={'id':'c','op':'mul','a':[oracle.hex_bits(2)]*2,'b':[oracle.hex_bits(3)]*2,'expect':'accept'}
        output={'schema':oracle.PROBE_SCHEMA,'id':'c','op':'mul','a':case['a'],'b':case['b'],'result':[oracle.hex_bits(-6)]*2}
        with self.assertRaises(Refusal):oracle.verify_probe_observation(case,output)


class StokesNative(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('RHEON_ALIGNED_STOKES_NATIVE'),'native step example not configured')
    def test_actual_step_and_hostile_certificates(self):
        with tempfile.TemporaryDirectory() as d:
            result=oracle.qualify_native(os.environ['RHEON_ALIGNED_STOKES_NATIVE'],Path(d))
            self.assertIn('unit-curl-sequence',result['native_cases'])
            original=oracle.read_json(Path(d)/'unit-curl.json')
            mutations=[]
            def add(name,fn):
                value=copy.deepcopy(original);fn(value);mutations.append((name,value))
            add('B excludes actual bound',lambda r:r['report'].__setitem__('bound',[oracle.hex_bits(0)]*2))
            add('step product false zero',lambda r:r['report'].__setitem__('step_product',[oracle.hex_bits(0)]*2))
            add('coefficient sign',lambda r:r['rows'][0]['terms'][0].__setitem__(1,oracle.hex_bits(-oracle.scalar(r['rows'][0]['terms'][0][1]))))
            add('coefficient magnitude',lambda r:r['rows'][0]['terms'][0].__setitem__(1,oracle.hex_bits(2*oracle.scalar(r['rows'][0]['terms'][0][1]))))
            add('positive weight changed',lambda r:r['rows'][0].__setitem__('weight',oracle.hex_bits(4)))
            add('positive mass changed',lambda r:r['active'][0].__setitem__('mass',oracle.hex_bits(2)))
            add('pressure area changed',lambda r:r['active'][0].__setitem__('area',oracle.hex_bits(2)))
            add('false energy zero',lambda r:r['report'].__setitem__('viscous_energy_delta',[oracle.hex_bits(0)]*2))
            add('zero-row deleted',lambda r:r['rows'].pop(next(i for i,x in enumerate(r['rows']) if not x['terms'])))
            add('mass sign',lambda r:r['active'][0].__setitem__('mass',oracle.hex_bits(-1)))
            add('caller wall speed',lambda r:r['final'][0].__setitem__(0,oracle.hex_bits(1e-30)))
            add('gauge pressure',lambda r:r['pressure'].__setitem__(r['gauge_cells'][0],oracle.hex_bits(1)))
            add('unrecognized data',lambda r:r.__setitem__('decorative_override',True))
            add('defect false point',lambda r:r['report']['update'][0].__setitem__('defect',[oracle.hex_bits(1)]*2))
            add('scale false point',lambda r:r['report']['momentum'][0].__setitem__('scale',[oracle.hex_bits(1e20)]*2))
            add('negative allowance',lambda r:r['report']['update'][0].__setitem__('allowance',[oracle.hex_bits(-1)]*2))
            add('counter unchanged',lambda r:r.__setitem__('state_after',r['state_before']))
            zero=next(i for i,x in enumerate(original['rows']) if not x['terms'])
            add('zero row made nonzero',lambda r:r['rows'][zero].__setitem__('terms',[[0,oracle.hex_bits(1)]]))
            for label,bad in mutations:
                with self.assertRaises(Refusal,msg=label):oracle.verify_record(bad,compute_projection=False)


if __name__=='__main__':unittest.main()
