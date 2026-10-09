"""Render actual native before/after states; no extra trajectory integration."""
import hashlib
import json
from pathlib import Path
import sys
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.ticker import MaxNLocator
from mpl_toolkits.mplot3d.art3d import Poly3DCollection
import numpy as np

def render(source, output):
    source,output=Path(source).resolve(),Path(output).resolve()
    if output.exists() or output.is_relative_to(Path(__file__).resolve().parents[1]):
        raise ValueError("fresh output outside Git required")
    output.mkdir(parents=True)
    names=["capped_axis","cancel_axis","spin_transfer","edge"]
    fig=plt.figure(figsize=(12,15),facecolor="#f5f7fa")
    hashes={}
    for row,name in enumerate(names):
        path=source/(name+".native.json");raw=json.loads(path.read_text());hashes[path.name]=hashlib.sha256(path.read_bytes()).hexdigest()
        if raw["error"] or raw["result"]["impact"] is None:raise ValueError("accepted impact needed")
        center=np.asarray(raw["after"]["center"]);point=np.asarray(raw["result"]["hit"]["point"]);radius=raw["radius_m"]
        before=np.asarray(raw["before"]["center"])
        lo=np.minimum(np.minimum(center,before),point)-radius*1.5
        hi=np.maximum(np.maximum(center,before),point)+radius*1.5
        for column,phase in enumerate(["before","after"]):
            ax=fig.add_subplot(4,2,row*2+column+1,projection="3d")
            frame=raw[phase];c=np.asarray(frame["center"]);vertices=np.asarray(frame["vertices"])
            static=np.asarray(raw["static_vertices"])
            ax.add_collection3d(Poly3DCollection([static[t] for t in raw["static_triangles"]],facecolor="#cbd5e1",edgecolor="#64748b",alpha=.22))
            ax.add_collection3d(Poly3DCollection([vertices[t] for t in raw["moving_triangles"]],facecolor="#3182ce",edgecolor="#174a7a",alpha=.65))
            theta,phi=np.meshgrid(np.linspace(0,2*np.pi,18),np.linspace(0,np.pi,10))
            ax.plot_wireframe(c[0]+radius*np.cos(theta)*np.sin(phi),c[1]+radius*np.sin(theta)*np.sin(phi),c[2]+radius*np.cos(phi),color="#60a5fa",alpha=.25,linewidth=.5)
            velocity=np.asarray(frame["velocity"]);omega=np.asarray(frame["omega"])
            ax.quiver(*c,*(.08*velocity),color="#dc2626",arrow_length_ratio=.2)
            ax.quiver(*c,*(.04*omega),color="#7c3aed",arrow_length_ratio=.2)
            ax.plot(*np.stack([before,center]).T,color="#64748b",linestyle=":")
            if phase=="after":
                ax.scatter(*point,color="#166534",s=20)
                impulse=np.asarray(raw["result"]["impact"]["impulse_n_s"])
                ax.quiver(*point,*(.03*impulse),color="#166534",arrow_length_ratio=.2)
            ax.set_xlim(lo[0],hi[0]);ax.set_ylim(lo[1],hi[1]);ax.set_zlim(lo[2],hi[2]);ax.set_box_aspect(hi-lo)
            ax.view_init(22,-65);ax.set_xlabel("x / m");ax.set_ylabel("y / m");ax.set_zlabel("z / m")
            ax.set_title(f"{name.replace('_',' ')}: actual {phase}, t={frame['time_s']:.6g} s",fontsize=11)
            for axis in [ax.xaxis,ax.yaxis,ax.zaxis]:axis.set_major_locator(MaxNLocator(3))
            text=f"V={np.array2string(velocity,precision=4)} m/s\nω={np.array2string(omega,precision=4)} rad/s"
            if phase=="after":
                impact=raw["result"]["impact"]
                text+=f"\n{impact['candidate']}; unused={raw['result']['unused_interval_s']:.6g} s\nΔT={impact['kinetic_after_j']-impact['kinetic_before_j']:.6g} J"
            ax.text2D(.02,.97,text,transform=ax.transAxes,fontsize=9,va="top",
                      bbox=dict(facecolor="white",edgecolor="none",alpha=.9,pad=3))
    fig.suptitle("Isolated Coulomb sphere impact — actual native states",fontsize=17)
    fig.text(.5,.02,"Blue: retained mesh + declared sphere; red: V × 0.08 s; purple: ω × 0.04 m·s; green: J × 0.03 m/(N·s).\nStatic finite facets use actual vertices; view clips distant geometry. No post-impact trajectory is integrated.",ha="center",fontsize=10)
    fig.subplots_adjust(top=.94,bottom=.09,hspace=.32,wspace=.12)
    path=output/"isolated-sphere-friction.jpg";fig.savefig(path,dpi=130,pil_kwargs={"quality":85});plt.close(fig)
    receipt={"source_native_sha256":hashes,"image_sha256":hashlib.sha256(path.read_bytes()).hexdigest(),"scope":"actual native before and accepted after; single impact STOP; arrows explicitly scaled; no renderer physics"}
    (output/"render-receipt.json").write_text(json.dumps(receipt,indent=2)+"\n")
    print(json.dumps(receipt,indent=2))
if __name__=="__main__":render(*sys.argv[1:])
