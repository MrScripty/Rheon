"""Render actual stored native endpoints, finite facets and declared colliders."""
import hashlib
import json
from pathlib import Path
import sys
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main(source,output):
    source,output=Path(source),Path(output)
    if output.exists():raise ValueError("fresh output required")
    output.mkdir(parents=True)
    receipt=json.loads((source/"receipt.json").read_text())
    names=["face","edge","vertex","oblique_edge"]
    fig=plt.figure(figsize=(16,12),facecolor="#101827")
    datahashes={}
    for slot,name in enumerate(names,1):
        file=source/(name+".native.json");raw=json.loads(file.read_text());datahashes[file.name]=sha(file)
        expected=next(c["native_sha256"] for c in receipt["cases"] if c["name"]==name)
        if sha(file)!=expected:raise ValueError("native data differs from qualification")
        if raw["error"] is not None or raw["result"]["hit"] is None:raise ValueError("accepted native contact required")
        ax=fig.add_subplot(2,2,slot,projection="3d",facecolor="#101827")
        static=np.asarray(raw["static_vertices"]);tris=raw["static_triangles"]
        ax.add_collection3d(Poly3DCollection([static[t] for t in tris],facecolors="#4b718f",edgecolors="#9fc5e8",alpha=.5))
        u=np.linspace(0,2*np.pi,25);v=np.linspace(0,np.pi,13)
        sph=np.array([np.outer(np.cos(u),np.sin(v)),np.outer(np.sin(u),np.sin(v)),np.outer(np.ones_like(u),np.cos(v))])
        for label,color in [("before","#52b8ff"),("after","#ffca63")]:
            state=raw[label];c=np.asarray(state["center"]);points=np.asarray(state["vertices"])
            ax.add_collection3d(Poly3DCollection([points[t] for t in raw["moving_triangles"]],facecolors=color,edgecolors=color,alpha=.45))
            # Declared analytic collider at ACTUAL stored COM; no pose integration.
            surface=sph*raw["radius_m"]+c[:,None,None]
            ax.plot_wireframe(*surface,color=color,linewidth=.5,alpha=.5,rstride=2,cstride=2)
            ax.quiver(*c,*state["velocity"],length=.3,normalize=False,color=color,arrow_length_ratio=.15)
        before=np.asarray(raw["before"]["center"]);after=np.asarray(raw["after"]["center"])
        ax.plot(*np.vstack([before,after]).T,color="#e7edf6",linestyle="--",linewidth=1.3)
        hit=raw["result"]["hit"];p=np.asarray(hit["point"]);n=np.asarray(raw["result"]["impact"]["normal"])
        ax.scatter(*p,color="#ff5a7c",s=35);ax.quiver(*p,*n,length=.7,color="#ff5a7c",arrow_length_ratio=.15)
        allp=np.vstack([static,before,after]);low=allp.min(0)-.8;high=allp.max(0)+.8
        ax.set_xlim(low[0],high[0]);ax.set_ylim(low[1],high[1]);ax.set_zlim(low[2],high[2]);ax.set_box_aspect(high-low)
        ax.set_xlabel("world x (m)",color="white");ax.set_ylabel("world y (m)",color="white");ax.set_zlabel("world z (m)",color="white")
        ax.tick_params(colors="#ced7e6",labelsize=7);ax.view_init(elev=23,azim=-62)
        ax.set_title(f"{name.replace('_',' ')} · {hit['feature']} · t={raw['after']['time_s']:.6f} s",color="white",pad=8)
    fig.suptitle("Declared sphere collider against actual finite static triangles",color="white",fontsize=18,y=.975)
    fig.text(.5,.935,"Blue: stored start   Gold: stored post-impact event   Red: actual contact normal",ha="center",color="#ced7e6",fontsize=11)
    fig.text(.5,.028,"Wire sphere = declared collision shape; solid octahedron = retained moving render/traction mesh.\nOne frictionless impact, e=0.5, then stop. No remaining-time continuation or arbitrary moving-mesh CCD.",ha="center",color="#ced7e6",fontsize=10)
    fig.subplots_adjust(left=.02,right=.98,top=.89,bottom=.10,wspace=.03,hspace=.12)
    image=output/"finite-static-sphere-contact.jpg";fig.savefig(image,dpi=120,pil_kwargs={"quality":85});plt.close(fig)
    record={"image":image.name,"image_sha256":sha(image),"native_data_sha256":datahashes,
            "oracle_receipt_sha256":sha(source/"receipt.json"),"native_executable_sha256":receipt["executable_sha256"],
            "geometry":"actual stored native endpoints and indexed triangles; declared analytic collider at stored COM",
            "display_pose_integration":False}
    (output/"render-receipt.json").write_text(json.dumps(record,indent=2)+"\n")
if __name__=="__main__":main(*sys.argv[1:])
