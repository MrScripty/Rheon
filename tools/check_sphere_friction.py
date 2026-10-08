"""Fresh independent convex-distance events and tangent-disk energy minimization.

Compares actual native pose/mesh, translation/spin impulses and diagnostics.
Uses 80-digit arithmetic, exact Fraction axis specimens, strict schemas and
negative probes active under python -O. No global or IEEE accuracy enclosure.
"""
import argparse
import copy
from fractions import Fraction as F
import hashlib
import json
import math
from pathlib import Path
import subprocess
import mpmath as mp
import check_sphere_contact as contact
from check_sphere_contact import require, num, vector, dot, length, keys

mp.mp.dps = 80
TOLERANCE = mp.mpf("2e-12")
VECTORS = {"contact_lever_m", "lever_normal_cross_m", "point_velocity_before_m_s",
           "point_velocity_after_m_s", "tangent_velocity_before_m_s", "tangent_velocity_after_m_s",
           "tangent_impulse_n_s", "impulse_n_s", "angular_impulse_n_m_s", "momentum_defect_n_s",
           "spin_momentum_defect_n_m_s", "world_angular_defect_n_m_s", "tangent_law_defect_m_s"}
SCALARS = {"represented_contact_distance_m", "slip_before_m_s", "slip_after_m_s",
           "inverse_tangent_mass_kg_inv", "normal_impulse_n_s", "cancellation_impulse_n_s",
           "coulomb_cap_n_s", "tangent_magnitude_n_s", "point_normal_coupling_defect_m_s",
           "tangent_impulse_normal_defect_n_s", "cone_excess_n_s", "restitution_defect_m_s",
           "kinetic_before_j", "kinetic_after_j", "predicted_energy_change_j", "point_midpoint_work_j",
           "energy_defect_j", "work_defect_j"}

def cross(a,b):
    return mp.matrix([a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]])
def tangent(v,n):return v-dot(v,n)*n
def energy(m,i,v,w):return m*dot(v,v)/2+i*dot(w,w)/2

def fixtures():
    out=contact.cases()
    refusal={"InitialContact":"Contact(InitialContact { triangle: 0 })",
             "Ambiguous":"Contact(Ambiguous { triangle: 0 })",
             "Simultaneous":"Contact(Simultaneous { first: 0, second: 1 })",
             "RadiusInertiaMismatch":"Contact(RadiusInertiaMismatch)",
             "RotationLimit":"Contact(Motion(RotationLimit))",
             "InvalidDuration":"Contact(InvalidDuration)"}
    for c in out:
        c["mu"]=.2
        if c.get("error"):c["exact_error"]=refusal[c["error"]]
    def axis(name,v,w=(0.,0.,0.),mu=.1,e=1.,**changes):
        c=dict(name=name,c=[0.,0.,.5],v=list(v),w=list(w),mu=mu,e=e,h=.2,
               radius=.25,mass=2.,inertia=.05,vertices=[[-4.,-4.,0.],[4.,-4.,0.],[0.,4.,0.]],triangles=[[0,1,2]])
        c.update(changes);return c
    out += [axis("capped_axis",[2.,0.,-2.]),axis("cancel_axis",[2.,0.,-2.],mu=.5),
            axis("zero_mu_axis",[2.,0.,-2.],mu=0.),axis("e0_axis",[2.,0.,-2.],mu=.5,e=0.),
            axis("spin_transfer",[0.,0.,-2.],[0.,1.,0.],mu=.5),
            axis("spin_transfer_capped",[0.,0.,-2.],[0.,1.,0.],mu=.01),
            axis("axial_spin",[2.,0.,-2.],[0.,0.,1.],mu=.5),
            axis("mixed_spin",[.3,-.7,-2.],[.8,-.6,.3],mu=.3),
            axis("rounded_small_slip",[1.,0.,-4.],[0.,4.,0.],mu=.5,h=.1),
            axis("exact_zero_slip",[0.,0.,-2.],mu=.5),
            axis("tiny_nonzero_slip",[2**-40,0.,-2.],mu=.5),
            axis("tie",[7.,0.,-2.],mu=.5),
            axis("tie_below",[7.,0.,-2.],mu=math.nextafter(.5,0.)),
            axis("tie_above",[7.,0.,-2.],mu=math.nextafter(.5,1.)),
            axis("invalid_mu",[2.,0.,-2.],mu=-.1,error="InvalidCoefficient",exact_error="InvalidCoefficient"),
            axis("cap_overflow",[2.,0.,-2.],mu=float.fromhex('0x1.fffffffffffffp+1023'),error="ArithmeticFailure",exact_error="Contact(ArithmeticFailure)"),
            axis("cap_underflow",[2.,0.,-2.],mu=float.fromhex('0x0.0000000000001p-1022'),e=0.,mass=.125,inertia=.003125,error="ArithmeticFailure",exact_error="Contact(ArithmeticFailure)"),
            axis("product_underflow",[2**-536,0.,-2.],mu=.5,inertia=.01,error="ArithmeticFailure",exact_error="Contact(ArithmeticFailure)")]
    # Spin is an axial vector: an improper reflection needs det(Q)*Q omega.
    base=axis("oblique_spin",[.3,-.7,-2.],[.8,-.6,.3],mu=.3)
    transforms=[("oblique_spin",lambda p:[p[0],.6*p[1]-.8*p[2],.8*p[1]+.6*p[2]],1),
                ("reflected_spin",lambda p:[-p[0],p[1],p[2]],-1)]
    for name,transform,det in transforms:
        c=copy.deepcopy(base);c["name"]=name
        for k in ["c","v"]:c[k]=transform(c[k])
        c["w"]=[det*x for x in transform(c["w"])]
        c["vertices"]=[transform(p) for p in c["vertices"]];out.append(c)
    return out

def native_input(c):
    values=[c["radius"],c["e"],c["mu"],c["h"],c["mass"],c["inertia"],*c["c"],*c["v"],*c["w"],len(c["vertices"]),len(c["triangles"])]
    values += [x for p in c["vertices"] for x in p]+[i for t in c["triangles"] for i in t]
    return " ".join(map(str,values))+"\n"

def disk_oracle(c,event):
    if event is None or event["hit"] is None:return None
    m,i=num(c["mass"]),num(c["inertia"])
    v,w=vector(c["v"]),vector(c["w"])
    r=event["point"]-event["center"];n=event["normal"]
    g=v+cross(w,r);vt=tangent(g,n);s=length(vt)
    # Evaluate the quadratic energy at its interval endpoints and stationary
    # point; choose the minimum. Geometry oracle is independent convex distance.
    k=1/m+dot(r,r)/i;cap=num(c["mu"])*length(event["impulse"])
    choices=[mp.mpf(0),cap]
    if 0<=s/k<=cap:choices.append(s/k)
    qt=min(choices,key=lambda q:-s*q+k*q*q/2)
    jt=-qt*vt/s if s else mp.zeros(3,1)
    j=event["impulse"]+jt
    va=v+j/m;wa=w+cross(r,j)/i
    return {"velocity":va,"omega":wa,"impulse":j,"tangent":jt,"angular":cross(r,j),
            "qt":qt,"energy":energy(m,i,va,wa)-energy(m,i,v,w)}

def evaluate(raw,c,event,expected):
    keys(raw,contact.TOP|{"coulomb_coefficient"})
    require(num(raw["coulomb_coefficient"])==num(c["mu"]),"pinned coefficient")
    contact.frame_schema(raw["before"]);contact.frame_schema(raw["after"])
    # Reject bools before numeric equality can let True stand in for 1.
    for p in raw["static_vertices"]:vector(p)
    for tri in raw["static_triangles"]+raw["moving_triangles"]:
        require(type(tri) is list and len(tri)==3 and all(type(x) is int for x in tri),"indices")
    normal_raw=copy.deepcopy(raw);normal_raw.pop("coulomb_coefficient")
    if c.get("error"):
        require(raw["error"]==c["exact_error"] and raw["result"] is None,"exact refusal identity")
        return contact.evaluate(normal_raw,c,None)
    require(raw["error"] is None,"unexpected refusal: "+str(raw["error"]))
    result=raw["result"];keys(result,contact.RESULT|{"normal_proposal"})
    impact=result["impact"]
    normal_raw["result"]["impact"]=normal_raw["result"].pop("normal_proposal")
    if expected is not None:
        keys(impact,VECTORS|SCALARS|{"candidate"})
        for key in VECTORS:vector(impact[key])
        for key in SCALARS:num(impact[key])
        require(type(impact["candidate"]) is str and impact["candidate"] in {"NoTangentialImpulse","SlipCancellation","CoulombCapped"},"computed candidate")
        # Existing geometry/pose/normal-proposal checker sees an unpublished
        # normal intermediate constructed only for its scope. The actual final
        # native velocity/spin are checked separately below, never overwritten.
        normal_raw["after"]["velocity"]=[float(x) for x in event["velocity"]]
        normal_raw["after"]["omega"]=c["w"]
    else:
        require(impact is None and result["normal_proposal"] is None,"miss has no impulses")
    legacy=contact.evaluate(normal_raw,c,event)
    errors=[legacy["maximum_algebra_error"]]
    physical=dict(legacy["physical_errors"],omega_rad_s=0.)
    def compare(a,b,kind=None):
        actual=a if isinstance(a,mp.mpf) else num(a)
        error=abs(actual-b);require(error<=TOLERANCE,"friction comparison: "+str(error));errors.append(float(error))
        if kind:physical[kind]=max(physical.get(kind,0.),float(error))
    def cv(a,b,kind=None):
        vector(a)
        for j in range(3):compare(a[j],b[j],kind)
    if expected is not None:
        for key,ek,kind in [("velocity","velocity","velocity_m_s"),("omega","omega","omega_rad_s")]:cv(raw["after"][key],expected[ek],kind)
        cv(impact["impulse_n_s"],expected["impulse"]);cv(impact["tangent_impulse_n_s"],expected["tangent"])
        cv(impact["angular_impulse_n_m_s"],expected["angular"])
        compare(impact["tangent_magnitude_n_s"],expected["qt"])
        compare(num(impact["kinetic_after_j"])-num(impact["kinetic_before_j"]),expected["energy"])
        m,i=num(c["mass"]),num(c["inertia"]);v,w=vector(c["v"]),vector(c["w"])
        va,wa=vector(raw["after"]["velocity"]),vector(raw["after"]["omega"])
        center=vector(raw["after"]["center"]);point=vector(result["hit"]["point"]);r=point-center
        n=vector(result["normal_proposal"]["normal"]);g=v+cross(w,r);ga=va+cross(wa,r)
        vt,vta=tangent(g,n),tangent(ga,n);s=length(vt);k=1/m+dot(r,r)/i
        jn=-(1+num(c["e"]))*m*dot(v,n);cap=num(c["mu"])*jn
        # Labels describe *stored computed* candidates. Physical reference
        # does not certify a real branch at adjacent/tiny floating boundaries.
        stop=num(impact["cancellation_impulse_n_s"]);stored_cap=num(impact["coulomb_cap_n_s"])
        mode="NoTangentialImpulse" if impact["slip_before_m_s"]==0 or c["mu"]==0 else ("SlipCancellation" if stop<=stored_cap else "CoulombCapped")
        require(impact["candidate"]==mode,"computed branch tie policy")
        # Independently pinned axis boundaries supplement the absolute physical
        # comparison. A perturbation below that tolerance must not forge modes.
        if c["name"] in {"tie","tie_below","tie_above"}:
            pinned="CoulombCapped" if c["name"]=="tie_below" else "SlipCancellation"
            require(mode==pinned and stop==4 and stored_cap==num(c["mu"])*8,"exact axis boundary and adjacent modes")
        if c["name"]=="tiny_nonzero_slip":
            require(mode=="SlipCancellation" and impact["slip_before_m_s"]>0,"tiny positive slip remains positive branch")
        if c["name"]=="exact_zero_slip":
            require(mode=="NoTangentialImpulse" and impact["slip_before_m_s"]==0,"exact axis zero slip")
        qt=num(impact["tangent_magnitude_n_s"]);jt=vector(impact["tangent_impulse_n_s"]);j=vector(impact["impulse_n_s"]);angular=cross(r,j)
        require(impact["represented_contact_distance_m"]>0 and impact["inverse_tangent_mass_kg_inv"]>0 and impact["normal_impulse_n_s"]>0,"positive response quantities")
        require(stop>=0 and stored_cap>=0 and impact["slip_before_m_s"]>=0 and impact["slip_after_m_s"]>=0,"nonnegative disk candidates")
        if mode=="NoTangentialImpulse":
            require(qt==0 and all(x==0 for x in jt),"explicit zero impulse branch")
        else:
            require(qt>0 and qt==min(stop,stored_cap),"exact stored candidate selection, including tiny positive impulses")
        values={"contact_lever_m":r,"lever_normal_cross_m":cross(r,n),"point_velocity_before_m_s":g,
                "point_velocity_after_m_s":ga,"tangent_velocity_before_m_s":vt,"tangent_velocity_after_m_s":vta,
                "momentum_defect_n_s":m*(va-v)-j,"spin_momentum_defect_n_m_s":i*(wa-w)-angular,
                "world_angular_defect_n_m_s":cross(center,m*(va-v))+i*(wa-w)-cross(point,j),
                "tangent_law_defect_m_s":vta-vt-k*jt}
        for key,value in values.items():cv(impact[key],value)
        kb,ka=energy(m,i,v,w),energy(m,i,va,wa)
        predicted=-m/2*(1-num(c["e"])**2)*dot(v,n)**2-qt*s+k*qt**2/2
        work=dot((g+ga)/2,j)
        scalar={"represented_contact_distance_m":length(r),"slip_before_m_s":s,"slip_after_m_s":length(vta),
                "inverse_tangent_mass_kg_inv":k,"normal_impulse_n_s":jn,"cancellation_impulse_n_s":s/k,
                "coulomb_cap_n_s":cap,"point_normal_coupling_defect_m_s":dot(g,n)-dot(v,n),
                "tangent_impulse_normal_defect_n_s":dot(jt,n),"cone_excess_n_s":max(mp.mpf(0),length(jt)-cap),
                "restitution_defect_m_s":dot(va,n)+num(c["e"])*dot(v,n),"kinetic_before_j":kb,"kinetic_after_j":ka,
                "predicted_energy_change_j":predicted,"point_midpoint_work_j":work,"energy_defect_j":ka-kb-predicted,
                "work_defect_j":ka-kb-work}
        for key,value in scalar.items():compare(impact[key],value)
    return {"maximum_algebra_error":max(errors),"physical_errors":physical,"refused":False}

def fraction_axes():
    records=[]
    for e in [F(0),F(1,2),F(1)]:
        for mu in [F(0),F(1,100),F(1,10),F(1,2)]:
            for vx,wy in [(F(2),F(0)),(F(0),F(1)),(F(1),F(4)),(F(-3),F(-2))]:
                m,I,d,vn=F(2),F(1,20),F(1,4),F(-2)
                slip=vx-wy*d;k=1/m+d*d/I;jn=-(1+e)*m*vn
                candidates=[F(0),mu*jn]
                if abs(slip)/k<=mu*jn:candidates.append(abs(slip)/k)
                q=min(candidates,key=lambda t:-abs(slip)*t+k*t*t/2)
                jt= -q*(1 if slip>0 else -1) if slip else F(0)
                va=vx+jt/m;wa=wy-d*jt/I;vna=-e*vn
                kb=m*(vx*vx+vn*vn)/2+I*wy*wy/2;ka=m*(va*va+vna*vna)/2+I*wa*wa/2
                require(ka-kb== -m*(1-e*e)*vn*vn/2-q*abs(slip)+k*q*q/2,"exact rational total energy")
                require(ka<=kb and abs(jt)<=mu*jn and (va-wa*d)*slip>=0,"exact rational disk/decay")
                records.append(dict(e=str(e),mu=str(mu),vx=str(vx),wy=str(wy),jt=str(jt),vx_after=str(va),wy_after=str(wa),energy_change=str(ka-kb)))
    return records

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--executable",type=Path,required=True);p.add_argument("--output",type=Path,required=True);a=p.parse_args()
    require(not a.output.exists() and not a.output.resolve().is_relative_to(Path(__file__).resolve().parents[1]),"fresh external output required");a.output.mkdir(parents=True)
    summaries=[];frozen=None;refusal_frozen=None;boundary_frozen=None;tiny_frozen=None
    for c in fixtures():
        text=native_input(c);run=subprocess.run([a.executable],input=text,text=True,capture_output=True,timeout=20)
        require(run.returncode==0,"native bridge failed: "+run.stderr)
        (a.output/(c["name"]+".input")).write_text(text);(a.output/(c["name"]+".native.json")).write_text(run.stdout)
        raw=json.loads(run.stdout);event=contact.oracle(c);expected=disk_oracle(c,event)
        result=evaluate(raw,c,event,expected);result["name"]=c["name"];summaries.append(result)
        (a.output/(c["name"]+".oracle.json")).write_text(json.dumps({k:[str(x) for x in v] if isinstance(v,mp.matrix) else str(v) for k,v in (expected or {}).items()},indent=2)+"\n")
        if c["name"]=="edge":frozen=(raw,c,event,expected)
        if c["name"]=="invalid_mu":refusal_frozen=(raw,c,event,expected)
        if c["name"]=="tie":boundary_frozen=(raw,c,event,expected)
        if c["name"]=="tiny_nonzero_slip":tiny_frozen=(raw,c,event,expected)
    raw,c,event,expected=frozen
    probes=[lambda x:x["result"]["impact"].update(candidate="CertifiedSticking"),
            lambda x:x["result"]["impact"].update(coulomb_cap_n_s=True),
            lambda x:x["result"]["impact"].update(angular_impulse_n_m_s=[0.,0.,0.]),
            lambda x:x["result"]["impact"].update(work_defect_j=42.),
            lambda x:x["result"]["impact"].update(tangent_magnitude_n_s=-1.),
            lambda x:x["result"]["impact"]["contact_lever_m"].__setitem__(0,True),
            lambda x:x["after"]["omega"].__setitem__(0,42.),lambda x:x["after"]["q"].__setitem__(0,True),
            lambda x:x["static_vertices"][0].__setitem__(0,True),lambda x:x["static_triangles"][0].__setitem__(0,False),
            lambda x:x["result"].update(unused_interval_s=0.),lambda x:x["result"].pop("normal_proposal")]
    for mutate in probes:
        bad=copy.deepcopy(raw);mutate(bad)
        try:evaluate(bad,c,event,expected)
        except ValueError:pass
        else:raise ValueError("forgery accepted")
    raw,c,event,expected=refusal_frozen
    for error in ["InvalidCoefficient forged","Contact(InvalidCoefficient)",True]:
        bad=copy.deepcopy(raw);bad["error"]=error
        try:evaluate(bad,c,event,expected)
        except ValueError:pass
        else:raise ValueError("refusal forgery accepted")
    raw,c,event,expected=boundary_frozen
    for changes in [dict(coulomb_cap_n_s=4.-1e-13,tangent_magnitude_n_s=4.-1e-13,candidate="CoulombCapped"),
                    dict(cancellation_impulse_n_s=4.+1e-13,candidate="CoulombCapped"),
                    dict(coulomb_cap_n_s=4.+1e-13)]:
        bad=copy.deepcopy(raw);bad["result"]["impact"].update(changes)
        try:evaluate(bad,c,event,expected)
        except ValueError:pass
        else:raise ValueError("axis boundary forgery accepted")
    raw,c,event,expected=tiny_frozen
    bad=copy.deepcopy(raw);bad["result"]["impact"].update(slip_before_m_s=0.,tangent_magnitude_n_s=0.,tangent_impulse_n_s=[0.,0.,0.],candidate="NoTangentialImpulse")
    try:evaluate(bad,c,event,expected)
    except ValueError:pass
    else:raise ValueError("tiny slip zero-impulse forgery accepted")
    fractions=fraction_axes();(a.output/"exact-rational-axis.json").write_text(json.dumps(fractions,indent=2)+"\n")
    receipt={"scope":"isolated fixed-contact isotropic sphere Coulomb impulse; no persistent force/global IEEE enclosure",
             "cases":summaries,"accepted":sum(not c["refused"] for c in summaries),"refused":sum(c["refused"] for c in summaries),
             "fraction_axis_specimens":len(fractions),"negative_probes":len(probes)+7,"algebra_tolerance":str(TOLERANCE),
             "maximum_algebra_error":max(c["maximum_algebra_error"] for c in summaries),
             "physical_errors":{k:max(c["physical_errors"].get(k,0.) for c in summaries) for k in ["time_s","point_m","normal","velocity_m_s","omega_rad_s"]},
             "executable_sha256":hashlib.sha256(a.executable.read_bytes()).hexdigest(),
             "source_sha256":{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),Path(contact.__file__),Path(__file__).with_name('check_rigid_motion.py')]}}
    (a.output/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    print(json.dumps({k:v for k,v in receipt.items() if k not in {"cases","source_sha256"}},indent=2))
if __name__=="__main__":main()
