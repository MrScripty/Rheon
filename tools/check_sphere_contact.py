"""Independent 80-digit closest-distance root oracle versus actual native events.

Uses bounded ternary minimization/bisection of distance to a convex triangle,
not the production seven-feature analytic root algorithm. Strict schema gates
remain active under python -O. Algebra tolerance is separate from physical
fixture error measurements and the API's unenclosed numerical policies.
"""
import argparse
import copy
import hashlib
import json
import math
from pathlib import Path
import subprocess
import mpmath as mp
from check_rigid_motion import exponential, quaternion_matrix

mp.mp.dps = 80
ALGEBRA_TOLERANCE = mp.mpf("1e-12")
MOVING = [[0,2,4],[2,1,4],[1,3,4],[3,0,4],[2,0,5],[1,2,5],[3,1,5],[0,3,5]]
REFERENCE = [[1,0,0],[-1,0,0],[0,1,0],[0,-1,0],[0,0,1],[0,0,-1]]
TOP = {"radius_m","restitution","requested_interval_s","mass_kg","inertia_kg_m2",
       "static_vertices","static_triangles","moving_triangles","before","after","error","result",
       "collision_shape","moving_mesh_role","relative_tolerance","max_gap_residual_m","simultaneous_window_s","static_triangle_limit"}
FRAME = {"time_s","center","q","velocity","omega","generation","surface_version","vertices","peak_payload_bytes"}
RESULT = {"unused_interval_s","requested_event_dt_s","represented_elapsed_s","clock_defect_s",
          "rotation_increment_rad","quaternion_norm_defect","translation_defect_m","hit","impact"}
HIT = {"triangle","feature","parameter","center","point","normal","barycentric","gap_residual_m"}
IMPACT = {"normal","normal_norm_defect","gap_residual_m","normal_velocity_before_m_s","normal_velocity_after_m_s",
          "impulse_n_s","momentum_defect","restitution_defect_m_s","kinetic_before_j","kinetic_after_j",
          "predicted_energy_change_j","energy_defect_j"}
def require(condition, message):
    if not condition: raise ValueError(message)
def num(x):
    require(type(x) in (int,float) and math.isfinite(x), "finite scalar type")
    if type(x) is int: return mp.mpf(x)
    n,d=x.as_integer_ratio();return mp.mpf(n)/d
def vector(x):
    require(type(x) is list and len(x)==3,"vector shape")
    return mp.matrix([num(v) for v in x])
def dot(a,b):return sum(a[i]*b[i] for i in range(3))
def length(a):return mp.sqrt(dot(a,a))
def keys(obj, expected):require(type(obj) is dict and set(obj)==expected,"strict object keys")
def scalar_fields(obj, names):
    for k in names:num(obj[k])
def frame_schema(f):
    keys(f,FRAME)
    scalar_fields(f,["time_s"])
    for k in ["center","velocity","omega"]:vector(f[k])
    require(type(f["q"]) is list and len(f["q"])==4,"quaternion shape")
    for x in f["q"]:num(x)
    for k in ["generation","surface_version","peak_payload_bytes"]:
        require(type(f[k]) is int and f[k]>=0,"integer metadata")
    require(type(f["vertices"]) is list and len(f["vertices"])==6,"six retained vertices")
    for p in f["vertices"]:vector(p)
def closest(p, tri):
    # Independent convex QP: stationary barycentric interior, then all three
    # constrained edge minima. No production feature/Voronoi root formulas.
    a,b,c=tri;u=b-a;v=c-a;w=p-a
    uu,uv,vv=dot(u,u),dot(u,v),dot(v,v)
    det=uu*vv-uv*uv
    s=(vv*dot(w,u)-uv*dot(w,v))/det
    t=(uu*dot(w,v)-uv*dot(w,u))/det
    candidates=[]
    if s>=0 and t>=0 and s+t<=1:candidates.append(a+s*u+t*v)
    for x,y in [(a,b),(b,c),(c,a)]:
        e=y-x;q=max(mp.mpf(0),min(mp.mpf(1),dot(p-x,e)/dot(e,e)))
        candidates.append(x+q*e)
    return min(candidates,key=lambda q:dot(p-q,p-q))
def classify(point,tri):
    a,b,c=tri;u=b-a;v=c-a;w=point-a
    uu,uv,vv=dot(u,u),dot(u,v),dot(v,v);det=uu*vv-uv*uv
    s=(vv*dot(w,u)-uv*dot(w,v))/det;t=(uu*dot(w,v)-uv*dot(w,u))/det
    bary=[1-s-t,s,t]
    active=[i for i,x in enumerate(bary) if x>mp.mpf("1e-40")]
    if len(active)==3:feature="Face"
    elif len(active)==1:feature="Vertex("+str(active[0])+")"
    else:
        zero=next(i for i in range(3) if i not in active)
        feature="Edge("+str((zero+1)%3)+")"
    return feature,mp.matrix(bary)
def oracle(case):
    if case.get("error"):return None
    c,v=vector(case["c"]),vector(case["v"]);h=num(case["h"]);r=num(case["radius"])
    vertices=list(map(vector,case["vertices"]));hits=[]
    for ordinal,indices in enumerate(case["triangles"]):
        tri=[vertices[i] for i in indices]
        def residual(t):
            p=c+t*v;q=closest(p,tri);return dot(p-q,p-q)-r*r
        require(residual(0)>0,"oracle strict clearance")
        lo,hi=mp.mpf(0),h
        # Distance squared to a closed convex triangle is convex along a line.
        # Ternary minimization brackets an interior collision even if endpoints
        # both miss. Endpoints are also candidates (including exact end hits).
        for _ in range(260):
            x=(2*lo+hi)/3;y=(lo+2*hi)/3
            if residual(x)<residual(y):hi=y
            else:lo=x
        tm=min([mp.mpf(0),h,(lo+hi)/2],key=residual)
        rm=residual(tm)
        if rm>mp.mpf("1e-40"):continue
        if abs(rm)<mp.mpf("1e-40") and tm<h-mp.mpf("1e-30"):
            raise ValueError("oracle tangency needs explicit refusal fixture")
        lo,hi=mp.mpf(0),tm
        for _ in range(260):
            mid=(lo+hi)/2
            if residual(mid)>0:lo=mid
            else:hi=mid
        time=(lo+hi)/2;p=c+time*v;q=closest(p,tri);n=(p-q)/length(p-q)
        feature,bary=classify(q,tri)
        hits.append((time,ordinal,p,q,n,feature,bary))
    if not hits:return {"time":h,"hit":None}
    time,ordinal,c,q,n,feature,bary=min(hits,key=lambda x:x[0])
    vn=dot(v,n);e=num(case["e"]);m=num(case["mass"])
    J=-(1+e)*m*vn*n;after=v+J/m
    return {"time":time,"hit":ordinal,"center":c,"point":q,"normal":n,"velocity":after,
            "impulse":J,"energy_change":-m/2*(1-e*e)*vn*vn,"feature":feature,"barycentric":bary}
def cases():
    def scene(name,c,v,e=.5,**changes):
        out=dict(name=name,c=c,v=v,e=e,h=1.,radius=.5,mass=2.,inertia=.2,w=[0.,0.,0.],
                 vertices=[[0.,0.,0.],[4.,0.,0.],[0.,4.,0.]],triangles=[[0,1,2]])
        out.update(changes);return out
    accepted=[scene("face",[1.,1.,2.],[0.,0.,-2.],w=[.1,.2,0.]),
              scene("edge",[1.,-.3,2.],[0.,0.,-2.]),
              scene("vertex",[-.3,-.3,2.],[0.,0.,-2.]),
              scene("edge_parallel_plane",[1.,-2.,.3],[0.,2.,0.]),
              scene("vertex_parallel_plane",[-2.,-2.,.3],[2.,2.,0.]),
              scene("backside",[1.,1.,-2.],[0.,0.,2.]),
              scene("reversed_winding",[1.,1.,2.],[0.,0.,-2.],triangles=[[0,2,1]]),
              scene("inelastic",[1.,1.,2.],[0.,0.,-2.],e=0.),
              scene("elastic",[1.,1.,2.],[0.,0.,-2.],e=1.),
              scene("finite_miss",[5.,5.,2.],[0.,0.,-2.]),
              scene("stationary_center",[5.,5.,2.],[0.,0.,0.],w=[0.,0.,.1]),
              scene("end_contact",[1.,1.,2.5],[0.,0.,-2.]),
              scene("nonmidpoint_wall",[1.,1.,2.],[0.,0.,-2.],vertices=[[0.,0.,.1375],[4.,0.,.1375],[0.,4.,.1375]]),
              scene("cross_velocity",[.7,.9,2.],[.25,-.1,-2.]),
              scene("earliest_second_facet",[1.,1.,2.],[0.,0.,-2.],vertices=[[0.,0.,0.],[4.,0.,0.],[0.,4.,0.],[0.,0.,.6],[4.,0.,.6],[0.,4.,.6]],triangles=[[0,1,2],[3,4,5]])]
    for factor in [.125,8.]:
        base=copy.deepcopy(accepted[1]);base["name"]="scaled_edge_"+str(factor)
        for k in ["c","v"]:base[k]=[factor*x for x in base[k]]
        base["vertices"]=[[factor*x for x in p] for p in base["vertices"]]
        base["radius"]*=factor;base["inertia"]*=factor**2;accepted.append(base)
    # Oblique and reflected actual facets. Spherical shape/inertia are invariant.
    for label,transform in [("oblique",lambda p:[p[0],.6*p[1]-.8*p[2],.8*p[1]+.6*p[2]]),
                            ("reflected",lambda p:[-p[0],p[1],p[2]])]:
        base=copy.deepcopy(accepted[1]);base["name"]=label+"_edge"
        for k in ["c","v","w"]:base[k]=transform(base[k])
        base["vertices"]=[transform(p) for p in base["vertices"]];accepted.append(base)
    refused=[scene("initial_overlap",[1.,1.,.4],[0.,0.,-2.],error="InitialContact"),
             scene("initial_touch",[1.,1.,.5],[0.,0.,-2.],error="InitialContact"),
             scene("tangent_vertex",[-.3,-.4,2.],[0.,0.,-2.],error="Ambiguous"),
             scene("feature_partition",[1.,0.,2.],[0.,0.,-2.],error="Ambiguous"),
             scene("duplicate_simultaneous",[1.,1.,2.],[0.,0.,-2.],triangles=[[0,1,2],[0,1,2]],error="Simultaneous"),
             scene("distinct_normal_simultaneous",[2.,1.,2.],[-2.,0.,-2.],vertices=[[0.,0.,0.],[4.,0.,0.],[0.,4.,0.],[0.,0.,4.]],triangles=[[0,1,2],[0,2,3]],error="Simultaneous"),
             scene("radius_inertia",[1.,1.,2.],[0.,0.,-2.],inertia=1.,error="RadiusInertiaMismatch"),
             scene("spin_cap",[1.,1.,2.],[0.,0.,-2.],w=[0.,0.,.5],error="RotationLimit"),
             scene("zero_duration",[1.,1.,2.],[0.,0.,-2.],h=0.,error="InvalidDuration")]
    return accepted+refused
def input_text(c):
    values=[c["radius"],c["e"],c["h"],c["mass"],c["inertia"],*c["c"],*c["v"],*c["w"],
            len(c["vertices"]),len(c["triangles"])]
    values += [x for p in c["vertices"] for x in p]+[i for t in c["triangles"] for i in t]
    return " ".join(map(str,values))+"\n"
def evaluate(raw,c,expected):
    keys(raw,TOP);frame_schema(raw["before"]);frame_schema(raw["after"])
    require(raw["collision_shape"]=="declared_sphere" and raw["moving_mesh_role"]=="render_and_traction","explicit separate collider")
    for k,value in [("relative_tolerance",1e-12),("max_gap_residual_m",1e-10),("simultaneous_window_s",1e-10)]:
        require(num(raw[k])==num(value),"pinned numerical policy")
    require(type(raw["static_triangle_limit"]) is int and raw["static_triangle_limit"]==64,"triangle query cap")
    for k,ck in [("radius_m","radius"),("restitution","e"),("requested_interval_s","h"),("mass_kg","mass"),("inertia_kg_m2","inertia")]:
        require(num(raw[k])==num(c[ck]),"pinned physical input")
    require(raw["static_vertices"]==c["vertices"] and raw["static_triangles"]==c["triangles"] and raw["moving_triangles"]==MOVING,"pinned geometry")
    require(type(raw["static_vertices"]) is list and len(raw["static_vertices"])==len(c["vertices"]),"static vertex count")
    for p in raw["static_vertices"]:vector(p)
    for tri in raw["static_triangles"]+raw["moving_triangles"]:
        require(type(tri) is list and len(tri)==3 and all(type(x) is int for x in tri),"index types")
    errors=[];physical={"time_s":0.,"point_m":0.,"normal":0.,"velocity_m_s":0.}
    def compare(a,b,kind=None):
        error=abs(num(a)-b);require(error<=ALGEBRA_TOLERANCE,"algebra comparison: "+str(error))
        errors.append(float(error))
        if kind:physical[kind]=max(physical[kind],float(error))
    def compare_vec(a,b,kind=None):
        vector(a)
        for i in range(3):compare(a[i],b[i],kind)
    before,after=raw["before"],raw["after"]
    compare_vec(before["center"],vector(c["c"]));compare_vec(before["velocity"],vector(c["v"]));compare_vec(before["omega"],vector(c["w"]))
    require(before["q"]==[1.,0.,0.,0.] and before["time_s"]==0 and before["generation"]==2 and before["surface_version"]==4 and before["peak_payload_bytes"]==816,"pinned initial metadata")
    for p,axis in zip(before["vertices"],REFERENCE):compare_vec(p,vector(c["c"])+num(c["radius"])/2*mp.matrix(axis))
    if c.get("error"):
        require(type(raw["error"]) is str and c["error"] in raw["error"] and raw["result"] is None,"explicit refusal")
        require(after==before,"atomic refusal")
        return {"maximum_algebra_error":max(errors,default=0),"physical_errors":physical,"refused":True}
    require(raw["error"] is None,"unexpected native refusal: "+str(raw["error"]))
    result=raw["result"];keys(result,RESULT)
    scalar_fields(result,RESULT-{"hit","impact","translation_defect_m"});vector(result["translation_defect_m"])
    t=expected["time"];center=vector(c["c"])+t*vector(c["v"])
    compare(after["time_s"],t,"time_s");compare_vec(after["center"],center,"point_m")
    compare(result["requested_event_dt_s"],t,"time_s");compare(result["represented_elapsed_s"],num(after["time_s"])-num(before["time_s"]))
    compare(result["clock_defect_s"],num(result["represented_elapsed_s"])-num(result["requested_event_dt_s"]))
    compare(result["unused_interval_s"],num(c["h"])-t)
    actual_dt=num(result["requested_event_dt_s"])
    compare_vec(result["translation_defect_m"],
        vector(after["center"])-vector(before["center"])-actual_dt*vector(before["velocity"]))
    compare(result["quaternion_norm_defect"],mp.sqrt(sum(num(x)**2 for x in after["q"]))-1)
    require(after["generation"]==3 and after["surface_version"]==5 and after["peak_payload_bytes"]==816,"published metadata")
    compare_vec(after["omega"],vector(c["w"]))
    rotation=exponential(vector(c["w"]),t)
    actual_rotation=quaternion_matrix(after["q"])
    for i in range(3):
        for j in range(3):compare(float(actual_rotation[i,j]),rotation[i,j])
    for p,initial in zip(after["vertices"],before["vertices"]):
        compare_vec(p,center+rotation*(vector(initial)-vector(c["c"])),"point_m")
    compare(result["rotation_increment_rad"],t*length(vector(c["w"])))
    if expected["hit"] is None:
        require(result["hit"] is None and result["impact"] is None,"finite triangle miss")
        compare_vec(after["velocity"],vector(c["v"]),"velocity_m_s")
    else:
        hit,impact=result["hit"],result["impact"];keys(hit,HIT);keys(impact,IMPACT)
        require(type(hit["triangle"]) is int and hit["triangle"]==expected["hit"],"earliest actual facet")
        require(hit["feature"] in ["Face","Edge(0)","Edge(1)","Edge(2)","Vertex(0)","Vertex(1)","Vertex(2)"],"declared finite feature")
        require(hit["feature"]==expected["feature"],"independent closest-feature label")
        scalar_fields(hit,["parameter","gap_residual_m"])
        for k in ["center","point","normal","barycentric"]:vector(hit[k])
        scalar_fields(impact,IMPACT-{"normal","impulse_n_s","momentum_defect"})
        for k in ["normal","impulse_n_s","momentum_defect"]:vector(impact[k])
        compare(hit["parameter"],t/num(c["h"]));compare_vec(hit["center"],center,"point_m")
        compare_vec(hit["point"],expected["point"],"point_m");compare_vec(hit["normal"],expected["normal"],"normal")
        compare_vec(impact["normal"],expected["normal"],"normal");compare_vec(impact["impulse_n_s"],expected["impulse"])
        compare_vec(after["velocity"],expected["velocity"],"velocity_m_s")
        actualn=vector(impact["normal"]);v0=vector(before["velocity"]);v1=vector(after["velocity"])
        gap=length(vector(after["center"])-vector(hit["point"]))-num(c["radius"])
        compare(impact["gap_residual_m"],gap)
        compare(hit["gap_residual_m"],length(vector(hit["center"])-vector(hit["point"]))-num(c["radius"]))
        compare(impact["normal_norm_defect"],length(actualn)-1)
        vb,va=dot(v0,actualn),dot(v1,actualn)
        compare(impact["normal_velocity_before_m_s"],vb);compare(impact["normal_velocity_after_m_s"],va)
        compare(impact["restitution_defect_m_s"],va+num(c["e"])*vb)
        m,I=num(c["mass"]),num(c["inertia"]);w=vector(c["w"])
        kb=m/2*dot(v0,v0)+I/2*dot(w,w);ka=m/2*dot(v1,v1)+I/2*dot(w,w)
        compare(impact["kinetic_before_j"],kb);compare(impact["kinetic_after_j"],ka)
        compare(impact["predicted_energy_change_j"],expected["energy_change"])
        compare(impact["energy_defect_j"],ka-kb-num(impact["predicted_energy_change_j"]))
        compare_vec(impact["momentum_defect"],m*(v1-v0)-vector(impact["impulse_n_s"]))
        bary=vector(hit["barycentric"]);require(all(x>=0 for x in bary),"nonnegative barycentrics")
        compare_vec(hit["barycentric"],expected["barycentric"])
        compare(float(sum(bary)),mp.mpf(1))
        tri=[vector(c["vertices"][i]) for i in c["triangles"][hit["triangle"]]]
        compare_vec(hit["point"],sum((bary[i]*tri[i] for i in range(3)),mp.zeros(3,1)))
    return {"maximum_algebra_error":max(errors,default=0),"physical_errors":physical,"refused":False}
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("--executable",type=Path,required=True);p.add_argument("--output",type=Path,required=True);a=p.parse_args()
    require(not a.output.exists(),"fresh output required");a.output.mkdir(parents=True)
    summaries=[];frozen=None
    for case in cases():
        inp=input_text(case);result=subprocess.run([a.executable],input=inp,text=True,capture_output=True,timeout=20)
        require(result.returncode==0,"native bridge failure: "+result.stderr)
        (a.output/(case["name"]+".input")).write_text(inp);(a.output/(case["name"]+".native.json")).write_text(result.stdout)
        raw=json.loads(result.stdout);expected=oracle(case);summary=evaluate(raw,case,expected)
        summary["name"]=case["name"]
        summary["native_sha256"]=hashlib.sha256(result.stdout.encode()).hexdigest()
        summary["input_sha256"]=hashlib.sha256(inp.encode()).hexdigest()
        def portable(value):
            if isinstance(value,mp.matrix):return [str(x) for x in value]
            if isinstance(value,mp.mpf):return str(value)
            return value
        (a.output/(case["name"]+".oracle.json")).write_text(json.dumps(
            {k:portable(v) for k,v in expected.items()} if expected else {"declared_refusal":case["error"]},indent=2)+"\n")
        summaries.append(summary)
        if case["name"]=="edge":frozen=(raw,case,expected)
    raw,case,expected=frozen
    mutations=[lambda x:x["after"]["vertices"].pop(),lambda x:x["after"].update(generation=True),
        lambda x:x["result"]["hit"].update(feature="InfinitePlane"),lambda x:x["static_triangles"].append([0,1,2]),
        lambda x:x["after"]["velocity"].pop(),lambda x:x["before"]["vertices"][0].__setitem__(0,42.),
        lambda x:x.update(error="forged"),lambda x:x["result"]["impact"].update(kinetic_after_j=float("nan")),
        lambda x:x["result"]["hit"].update(feature="Face"),
        lambda x:x["result"].update(quaternion_norm_defect=42.),
        lambda x:x["result"].update(translation_defect_m=[42.,0.,0.])]
    for mutate in mutations:
        bad=copy.deepcopy(raw);mutate(bad)
        try:evaluate(bad,case,expected)
        except ValueError:pass
        else:raise ValueError("negative checker probe accepted")
    receipt={"oracle":"80-digit convex closest-distance minimization and bisection; Rodrigues matrix pose",
             "executable_sha256":hashlib.sha256(a.executable.read_bytes()).hexdigest(),"cases":summaries,
             "accepted":sum(not c["refused"] for c in summaries),"refused":sum(c["refused"] for c in summaries),
             "checker_negative_probes":len(mutations),"algebra_tolerance":str(ALGEBRA_TOLERANCE),
             "maximum_algebra_error":max(c["maximum_algebra_error"] for c in summaries),
             "physical_fixture_errors":{k:max(c["physical_errors"][k] for c in summaries) for k in summaries[0]["physical_errors"]},
             "accuracy_scope":"measured fixtures only; not global/IEEE error enclosure or arbitrary moving-mesh qualification"}
    receipt["source_sha256"]={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),Path(__file__).with_name("check_rigid_motion.py")]}
    receipt["mpmath_version"]=mp.__version__
    (a.output/"receipt.json").write_text(json.dumps(receipt,indent=2)+"\n");print(json.dumps(receipt,indent=2))
if __name__=="__main__":main()
