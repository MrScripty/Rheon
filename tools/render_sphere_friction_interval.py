"""Storyboard the ACTUAL native stored mesh/COM sequence; no display integrator."""
import hashlib,json,sys
from pathlib import Path
import numpy as np
import matplotlib
matplotlib.use('Agg')
import matplotlib.pyplot as plt
from mpl_toolkits.mplot3d.art3d import Poly3DCollection

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main(source,output):
    source,output=Path(source).resolve(),Path(output).resolve()
    root=Path(__file__).resolve().parents[1]
    if output.exists()or output.is_relative_to(root):raise ValueError('fresh external output')
    receipt=json.loads((source/'receipt.json').read_text());names=['repeated_dyadic_capped','cancellation_exact_end','friction_rotation_stop','rounded_departure_stop'];output.mkdir(parents=True)
    fig=plt.figure(figsize=(18,12),facecolor='#101827');datahashes={}
    u=np.linspace(0,2*np.pi,19);v=np.linspace(0,np.pi,11)
    sphere=np.array([np.outer(np.cos(u),np.sin(v)),np.outer(np.sin(u),np.sin(v)),np.outer(np.ones_like(u),np.cos(v))])
    for row,name in enumerate(names):
        file=source/(name+'.native.json');digest=sha(file)
        if receipt['evidence_sha256'][file.name]!=digest:raise ValueError('native receipt mismatch')
        raw=json.loads(file.read_text());datahashes[file.name]=digest
        if raw['admission_error']is not None:raise ValueError('accepted prefix needed')
        frames=[raw['before']]+[s['frame']for s in raw['sequence']]
        static=np.asarray(raw['static_vertices']);triangles=raw['static_triangles']
        status=raw['result']['status'];remaining=raw['result']['remaining_interval_s']
        fig.text(.02,.91-row*.218,f"{name.replace('_',' ')} · e={raw['restitution']:g}, μ={raw['coulomb_coefficient']:g} · {status} · remaining={remaining:.6g} s",color='white',fontsize=11)
        for column,frame in enumerate(frames):
            ax=fig.add_axes([.022+column*.193,.725-row*.218,.183,.145],projection='3d',facecolor='#101827')
            ax.add_collection3d(Poly3DCollection([static[t]for t in triangles],facecolors='#487690',edgecolors='#9bc1dd',alpha=.22))
            p=np.asarray(frame['vertices']);c=np.asarray(frame['center']);color='#5ec8ff'if column==0 else'#ffca63'
            ax.add_collection3d(Poly3DCollection([p[t]for t in raw['moving_triangles']],facecolors=color,edgecolors=color,alpha=.8))
            ax.plot_wireframe(*(sphere*raw['radius_m']+c[:,None,None]),color=color,linewidth=.5,alpha=.6,rstride=2,cstride=2)
            path=np.asarray([f['center']for f in frames[:column+1]]);ax.plot(*path.T,color='#e2edf7',linestyle='--',linewidth=1.)
            ax.quiver(*c,*frame['velocity'],length=.09,color=color,arrow_length_ratio=.2)
            label='stored start'
            if column:
                s=raw['sequence'][column-1];h=s['hit'];label=s['impact']['candidate']if h else'terminal coast'
                if h:
                    q=np.asarray(h['point']);n=np.asarray(s['hit']['normal']);ax.scatter(*q,color='#ff637c',s=15);ax.quiver(*q,*(.025*np.asarray(s['impact']['impulse_n_s'])),color='#ff637c');ax.quiver(*c,*(.04*np.asarray(frame['omega'])),color='#b794f4')
            limits=[(-1.3,1.3),(-.45,.8),(-.45,.45)]
            for setter,lim in zip([ax.set_xlim,ax.set_ylim,ax.set_zlim],limits):setter(*lim)
            ax.set_box_aspect([b-a for a,b in limits]);ax.view_init(elev=20,azim=-65);ax.tick_params(colors='#c8d4df',labelsize=6)
            ax.set_title(f"{column}: {label}\nt={frame['time_s']:.6g} s · V={np.linalg.norm(frame['velocity']):.5g} m/s · ω={np.linalg.norm(frame['omega']):.4g} rad/s",color='white',fontsize=8,pad=0)
            for axis,label in [(ax.set_xlabel,'x m'),(ax.set_ylabel,'y m'),(ax.set_zlabel,'z m')]:axis(label,color='#c8d4df',fontsize=7,labelpad=0)
    fig.suptitle('Bounded Coulomb impacts: actual accepted native sequence',color='white',fontsize=18,y=.98)
    fig.text(.5,.935,'Wire sphere: declared collider   Solid octahedron: stored moving mesh   Arrows: V × 0.09 s, ω × 0.04 m·s, J × 0.025 m/(N·s)',color='#c8d4df',ha='center',fontsize=11)
    fig.text(.5,.025,'Each frame follows an actual atomic publication; trajectories connect stored COM endpoints. Facets are clipped for the close view.\nBudget and exact-departure stops retain a positive remainder. No pushout, resting manifold, loads or fluid coupling.',color='#c8d4df',ha='center',fontsize=10)
    target=output/'bounded-coulomb-impact-sequence.jpg';fig.savefig(target,dpi=120,pil_kwargs={'quality':85});plt.close(fig)
    (output/'render-receipt.json').write_text(json.dumps(dict(image=target.name,image_sha256=sha(target),native_data_sha256=datahashes,oracle_receipt_sha256=sha(source/'receipt.json'),native_executable_sha256=receipt['executable_sha256'],display_pose_integration=False,geometry='actual native stored vertices, COM and finite static triangles; analytic collider rendered at stored COM'),indent=2)+'\n')
if __name__=='__main__':main(*sys.argv[1:])
