"""Independent exact controls and actual native hostile tests; no stepping."""
import copy
from fractions import Fraction as F
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

import check_viscous_boundary_wrench as c
from aligned_strain_oracle import Geometry, Refusal


class RationalWrenchControls(unittest.TestCase):
    def test_unit_uniform_translation_has_analytic_generalized_drag(self):
        g,ref=c.fixture('unit-center');faces,rows,lifts=c.lift_rows(g,ref)
        u=[F(1) if f.axis==0 else F(0) for f in faces]
        m=c.exact_metrics(g,faces,rows,lifts,ref,u)
        # Four corners in each of xy and xz contribute 2 units of drag.
        # Eight normal outer cells per x side contribute 2 units each (normal weight2V).
        self.assertEqual(m['solid_wrench'],[16,0,0,0,0,0])
        self.assertEqual(m['outer_wrench'],[32,0,0,0,0,0])
        self.assertEqual(m['fluid_wrench'],[-48,0,0,0,0,0])
        self.assertEqual(m['dissipation'],48)
        self.assertEqual(m['force_work'],-48)
        self.assertEqual(m['balance_defect'],[0]*6)

    def test_all_corner_reflections_have_independent_coupled_patch(self):
        g,ref=c.fixture('unit-center');faces,rows,lifts=c.lift_rows(g,ref)
        groups={}
        for row,lift in zip(rows,lifts):
            if row.closure==2:groups.setdefault((row.family,row.p),[]).append((row,lift))
        self.assertEqual(len(groups),12)
        for (axes,_),group in groups.items():
            self.assertEqual(len(group),3)
            indices=sorted({i for row,_ in group for i,_ in row.terms})
            k=[[sum(row.weight*dict(row.terms).get(a,0)*dict(row.terms).get(b,0) for row,_ in group)
                for b in indices] for a in indices]
            self.assertEqual(k[0][0],2);self.assertEqual(k[1][1],2)
            self.assertEqual(abs(k[0][1]),1)
            self.assertEqual(k[0][1],k[1][0])
            for row,lift in group:
                for axis in axes:
                    self.assertEqual(lift.solid[axis],-sum(a for i,a in row.terms if faces[i].axis==axis))

    def test_flat_lift_restores_both_derivatives_and_is_not_point_traction(self):
        g=Geometry((4,4,4),(F(1),)*3,(F(0),)*3,(1,1,1),(3,3,3))
        ref=(F(2),)*3;faces,rows,lifts=c.lift_rows(g,ref)
        row,lift=next((r,l) for r,l in zip(rows,lifts) if r.closure==1 and r.family==(0,1) and faces[r.terms[0][0]].axis==0)
        i,a=row.terms[0];f=faces[i]
        wall=list(f.position);wall[1]=g.plane(1,row.p[1])
        naive=[-a*x for x in c.rigid_basis(0,wall,ref)]
        self.assertEqual(lift.solid[5]-naive[5],1)
        # Omitting the normal-trace derivative destroys rigid angular cancellation.
        u=[c.rigid_basis(f.axis,f.position,ref)[5] for f in faces]
        gamma=sum(a*u[i] for i,a in row.terms)
        self.assertEqual(gamma+lift.solid[5],0)
        self.assertNotEqual(gamma+naive[5],0)

    def test_ideal_common_rigid_null_modes_all_geometry(self):
        for case in ('unit-center','anisotropic','translated-nonmidpoint','translated-large','reference-shift'):
            with self.subTest(case=case):
                g,ref=c.fixture(case);faces,rows,lifts=c.lift_rows(g,ref)
                for row,lift in zip(rows,lifts):
                    for k in range(6):
                        residual=sum(a*c.rigid_basis(faces[i].axis,faces[i].position,ref)[k] for i,a in row.terms)+lift.solid[k]+lift.outer[k]
                        self.assertEqual(residual,0)

    def test_stationary_sampled_rotation_is_not_global_null(self):
        g,ref=c.fixture('unit-center');faces,rows,lifts=c.lift_rows(g,ref)
        u=[c.rigid_basis(f.axis,f.position,ref)[5] for f in faces]
        self.assertGreater(c.exact_metrics(g,faces,rows,lifts,ref,u)['dissipation'],0)
        local=[F.from_float(x) for x in c.expected_fields(g,faces,rows,ref)['compatible-local-rotation']]
        metrics=c.exact_metrics(g,faces,rows,lifts,ref,local)
        self.assertEqual(metrics['dissipation'],6)
        selected=[s for row,s in zip(rows,metrics['strains']) if row.family==(0,1) and row.p==(1,1,0)]
        self.assertEqual(selected,[0]*4)

    def test_full_fluid_affine_engineering_shear_cross_components(self):
        g,ref=c.fixture('unit-center');faces,rows,lifts=c.lift_rows(g,ref)
        field=c.expected_fields(g,faces,rows,ref)['affine-strain'];u=list(map(F.from_float,field))
        metrics=c.exact_metrics(g,faces,rows,lifts,ref,u)
        for row,strain in zip(rows,metrics['strains']):
            if row.closure==0:
                expected={(0,1):F(1,4),(0,2):F(0),(1,2):F(1,8)}[row.family]
                self.assertEqual(strain,expected)
        self.assertGreater(metrics['dissipation'],0)

    def test_zero_rows_keep_independent_outer_and_solid_traces(self):
        g,ref=c.fixture('unit-center');faces,rows,lifts=c.lift_rows(g,ref)
        zeros=[(r,l) for r,l in zip(rows,lifts) if not r.terms]
        self.assertEqual(len(zeros),6)
        for row,lift in zeros:
            self.assertEqual(row.family[0],row.family[1])
            self.assertEqual(lift.solid[row.family[0]],-lift.outer[row.family[0]])
            self.assertNotEqual(lift.solid[row.family[0]],0)

    def test_reference_translation_covariance_and_scaling(self):
        g,ref=c.fixture('unit-center');faces,rows,lifts=c.lift_rows(g,ref)
        u=[F.from_float(x) for x in c.expected_fields(g,faces,rows,ref)['crosscomponent']]
        m=c.exact_metrics(g,faces,rows,lifts,ref,u)
        shift=(F(7,4),F(-1),F(3,4));newref=tuple(a+b for a,b in zip(ref,shift))
        nf,nr,nl=c.lift_rows(g,newref);n=c.exact_metrics(g,nf,nr,nl,newref,u)
        for key in ('solid_wrench','outer_wrench','fluid_wrench'):
            fx,fy,fz=m[key][:3]
            cross=(shift[1]*fz-shift[2]*fy,shift[2]*fx-shift[0]*fz,shift[0]*fy-shift[1]*fx)
            self.assertEqual(n[key][:3],m[key][:3])
            self.assertEqual(n[key][3:],[t-d for t,d in zip(m[key][3:],cross)])
        for factor in (F(1,10**40),F(10**40)):
            scaled=c.exact_metrics(g,faces,rows,lifts,ref,[factor*x for x in u])
            self.assertEqual(scaled['dissipation'],factor**2*m['dissipation'])
            self.assertEqual(scaled['solid_wrench'],[factor*x for x in m['solid_wrench']])

    def test_continuum_polynomial_force_torque_exact_integrals(self):
        g,ref=c.fixture('unit-center')
        self.assertEqual(c.polynomial_integral((0,0,1,-2,1),F(0),F(1)),F(1,30))
        self.assertEqual(c.continuum_polynomial_wrench(g,ref),[0,0,0,0,0,F(-1,225)])
        self.assertEqual(c.continuum_polynomial_wrench(g,ref,F(225),F(1)),[0,F(-1,2),0,0,0,-1])
        self.assertEqual(c.continuum_polynomial_wrench(g,ref,F(225),F(1),(1,2,0)),[0,0,F(-1,2),-1,0,0])
        self.assertEqual(c.continuum_polynomial_wrench(g,ref,F(225),F(1),(2,0,1)),[F(-1,2),0,0,0,-1,0])
        ag=Geometry((3,3,3),(F(2),F(3),F(4)),(F(10),F(-7),F(3)),(1,1,1),(2,2,2),F(5),F(7))
        center=tuple((ag.plane(d,1)+ag.plane(d,2))/2 for d in range(3))
        expected=-ag.viscosity*F(2)**3*F(3)**3*F(4)**5*(F(2)**2+F(3)**2)/450
        self.assertEqual(c.continuum_polynomial_wrench(ag,center)[5],expected)
        moved=c.continuum_polynomial_wrench(ag,(center[0]+1,center[1],center[2]+2),F(225),F(1))
        base=c.continuum_polynomial_wrench(ag,center,F(225),F(1))
        self.assertEqual(moved[0:3],base[0:3]);self.assertEqual(moved[3],2*base[1]);self.assertEqual(moved[5],base[5]-base[1])

    def test_coarse_continuum_compatible_polynomials_expose_resolution_limit(self):
        g,ref=c.fixture('unit-center');faces,rows,lifts=c.lift_rows(g,ref)
        fields=c.expected_fields(g,faces,rows,ref)
        for name in ('polynomial-curl','polynomial-tilted-curl','polynomial-tilted-curl-yz','polynomial-tilted-curl-zx'):
            self.assertTrue(all(value==0 for value in fields[name]))
            discrete=c.exact_metrics(g,faces,rows,lifts,ref,list(map(F.from_float,fields[name])))
            self.assertEqual(discrete['solid_wrench'],[0]*6)
        self.assertNotEqual(c.continuum_polynomial_wrench(g,ref,F(225),F(1)),[0]*6)

    def test_native_json_read_caps_and_duplicate_parser(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'oversized.json';p.write_bytes(b' '*(2*1024*1024+1))
            with self.assertRaisesRegex(Refusal,'byte cap'):c.read_json(p)
            p.write_text('{"a":1,"a":2}')
            with self.assertRaisesRegex(Refusal,'duplicate'):c.read_json(p)
            p.write_text('{"a":NaN}')
            with self.assertRaisesRegex(Refusal,'nonfinite'):c.read_json(p)

    def test_diagnostic_tolerance_is_contribution_scaled_with_no_floor(self):
        with self.assertRaises(Refusal):c.diagnostic(F(-1,10**30),F(1,10**30),F(1,10**30),'tiny sign flip')
        with self.assertRaises(Refusal):c.diagnostic(F(1,10**300),F(0),F(0),'structural zero')
        # A canceled strain may have a rounded scatter residual; its budget is
        # built from upstream products before cancellation, in the same units.
        g,ref=c.fixture('anisotropic');faces,rows,lifts=c.lift_rows(g,ref,stored=True)
        u=list(map(F.from_float,c.expected_fields(g,faces,rows,ref)['compatible-local-rotation']))
        metrics=c.exact_metrics(g,faces,rows,lifts,ref,u)
        self.assertTrue(any(s>abs(v) for s,v in zip(metrics['scales']['force'],metrics['force'])))


class NativeWrenchObservations(unittest.TestCase):
    @unittest.skipUnless(os.environ.get('RHEON_VISCOUS_WRENCH_EXECUTABLE'),'actual native binary not configured')
    def test_fresh_native_exact_assembly_diagnostics_and_hostile_records(self):
        executable=Path(os.environ['RHEON_VISCOUS_WRENCH_EXECUTABLE']).resolve()
        with tempfile.TemporaryDirectory() as d:
            for case in ('unit-center','anisotropic','translated-large','reference-shift'):
                path=Path(d)/(case+'.json')
                process=subprocess.run([str(executable),case,str(path)],capture_output=True,text=True,timeout=90)
                self.assertEqual(process.returncode,0,process.stderr)
                self.assertEqual(c.verify(path,case)['fixture'],case)
                if case=='unit-center':
                    packet=c.read_json(path)
                    self.assertGreaterEqual(len(c.mutation_suite(packet)),20)
                    duplicate=Path(d)/'duplicate.json'
                    duplicate.write_bytes(path.read_bytes().replace(b'"schema":',b'"schema":"extra","schema":',1))
                    with self.assertRaisesRegex(Refusal,'duplicate'):c.verify(duplicate)
                    bad=copy.deepcopy(packet);bad['fields'][1]['values'][0]=True
                    with self.assertRaises(Refusal):c.verify_record(bad)


if __name__=='__main__':unittest.main()
