"""Independent polynomial identities; no numerical solve or native campaign."""
import unittest
from fractions import Fraction as F
from math import factorial
from flat_wall_physics_model import (B,Z,AMPLITUDE,polynomial,integral,
    continuum_reference,average_wall_weights,reference_flux_field,pack_exact_vectors)
from check_flat_wall_ritz import Model,traction

class PhysicsReference(unittest.TestCase):
    def test_beta_reference_and_z_factor(self):
        beta=F(factorial(6)**2,factorial(13))
        self.assertEqual(integral(B,F(0),F(1)),beta)
        for s in (F(1,8),F(1,3),F(1,2),F(3,4)):
            self.assertEqual(polynomial(B,s),s**6*(1-s)**6)
            self.assertEqual(polynomial(Z,s),s**6*(1-s)**6*(s+F(1,2)))
        self.assertEqual(continuum_reference(),([F(-1),F(0),F(0)],[F(0),-F(1,60),-F(1,2)]))

    def test_quadratic_average_exactness_on_unequal_intervals(self):
        for h1,h2 in ((F(1,4),F(1,2)),(F(1,4),F(3,4)),(F(3,10),F(4,5))):
            w1,w2=average_wall_weights(h1,h2)
            for a,b in ((F(3),F(0)),(F(0),F(7)),(F(-2),F(5))):
                u1=a*h1/2+b*h1*h1/3
                u2=a*(h1+h2)/2+b*(h1*h1+h1*h2+h2*h2)/3
                self.assertEqual(w1*u1+w2*u2,a)
            # Cubic error equals the integral of the Peano kernel times u'''=6.
            u1=h1**3/4;u2=(h1**3+h1*h1*h2+h1*h2*h2+h2**3)/4
            self.assertEqual(w1*u1+w2*u2,-h1*h2/2)

    def test_reference_curl_equals_true_face_averages_with_exact_area(self):
        # Independently integrate continuum velocity polynomial on each face.
        yp={3:F(4),4:F(-10),5:F(6)}
        bp={k-1:k*v for k,v in B.items()}
        for n in (6,9,12):
            model=Model(n);p=list(map(F,model.p));field=reference_flux_field(model,False)
            for (a,face),v in field.items():
                x,y,z=model.coordinate(a,face);zi=integral(Z,p[z]-1,p[z+1]-1)
                if a==0:
                    exact=AMPLITUDE*polynomial(B,p[x]-1)*integral(yp,p[y],p[y+1])*zi/((p[y+1]-p[y])*(p[z+1]-p[z]))
                else:
                    exact=-AMPLITUDE*integral(bp,p[x]-1,p[x+1]-1)*p[y]**4*(1-p[y])**2*zi/((p[x+1]-p[x])*(p[z+1]-p[z]))
                self.assertEqual(v,exact)

    def test_dyadic_reference_secant_bias_closed_form(self):
        for n in (6,12):
            model=Model(n);h=F(3,n);m=n//3
            bx=h*sum((polynomial(B,F(i,m)) for i in range(1,m)),F(0))
            spatial=2*AMPLITUDE*bx*integral(Z,F(0),F(1))
            f,t=traction(model,reference_flux_field(model,False),'p1')
            self.assertEqual(f[0],-spatial*(1-h)**4)
            self.assertEqual(t[2],(F(1,2)-h)*f[0])

    def test_exact_output_interning_is_lossless(self):
        v={'exact':['1/3','0','1/3'],'nearest':[1/3,0.,1/3]}
        p=pack_exact_vectors({'a':v,'b':v})
        self.assertEqual([p['exact_value_table'][i] for i in p['a']['exact_indices']],v['exact'])
        self.assertEqual(p['a'],p['b'])

if __name__=='__main__':unittest.main()
