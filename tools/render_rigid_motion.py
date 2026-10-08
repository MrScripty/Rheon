"""JPEG85 moving meshes from checked actual native vertices and pose clocks."""
import argparse,hashlib,json
from pathlib import Path

def render(evidence,output):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d.art3d import Poly3DCollection
    from PIL import Image
    root=Path(__file__).resolve().parents[1];evidence=Path(evidence).resolve();output=Path(output).resolve()
    if output.exists() or output.is_relative_to(root):raise ValueError('fresh external output required')
    receipt=json.loads((evidence/'qualification.json').read_text());name='moving-force-torque'
    if receipt.get('qualified') is not True:raise ValueError('qualified native moving trajectory required')
    raw=(evidence/(name+'.stdout.json')).read_bytes();record=next(r for r in receipt['fixtures'] if r['name']==name)
    if hashlib.sha256(raw).hexdigest()!=record['stdout_sha256']:raise ValueError('native trajectory hash mismatch')
    data=json.loads(raw);frames=[data['initial']]+[s['stored'] for s in data['steps']];indices=[0,4,8,16]
    all_points=[p for i in indices for p in frames[i]['vertices']];limits=[(min(p[d] for p in all_points)-.12,max(p[d] for p in all_points)+.12) for d in range(3)]
    fig=plt.figure(figsize=(13,9),facecolor='#f6f7fb')
    for panel,i in enumerate(indices):
        state=frames[i];ax=fig.add_subplot(2,2,panel+1,projection='3d',facecolor='#f6f7fb')
        facets=[[state['vertices'][j] for j in tri] for tri in data['triangles']];colors=['#d27a37' if j in (2,3) else '#6d9ecc' for j in range(12)]
        ax.add_collection3d(Poly3DCollection(facets,facecolors=colors,edgecolors='#2f4c68',alpha=.35,linewidths=.8))
        c=state['center_of_mass'];ax.scatter(*c,color='#a83226',s=30)
        v=state['velocity_m_s'];ax.quiver(*c,*(.2*x for x in v),color='#a83226',linewidth=2,arrow_length_ratio=.2)
        ax.plot(*zip(*(f['center_of_mass'] for f in frames[:i+1])),color='#a83226',linewidth=1.5)
        ax.set(xlim=limits[0],ylim=limits[1],zlim=limits[2],xlabel='x (m)',ylabel='y (m)',zlabel='z (m)');ax.view_init(elev=23,azim=-60)
        ax.set_title(f"Actual stored mesh at t = {state['time_s']:g} s\nCOM = ({c[0]:.4f}, {c[1]:.4f}, {c[2]:.4f}) m",fontsize=10)
    final=frames[-1];fmt=lambda v:'('+', '.join(f'{x:.5g}' for x in v)+')'
    fig.suptitle('Force-driven spherical body: actual translation and rotation',fontsize=17,y=.975)
    fig.text(.035,.115,'Orange material facets receive world traction (1.2, 0.3, 0.6) N/m²; mass 2 kg, declared spherical moments (1, 1, 1) kg m².\n'
             'Post-kick quaternion exponential drift; 16 accepted steps of 0.0625 s. All panels share world axes and show native stored vertices.',fontsize=10)
    fig.text(.035,.065,f"Final native V = {fmt(final['velocity_m_s'])} m/s; omega = {fmt(final['angular_velocity_rad_s'])} rad/s.\nFinal stored q [real,x,y,z] = {fmt(final['orientation'])}; COM path and 0.2 m per (m/s) velocity glyphs shown in red.",fontsize=10)
    fig.text(.035,.023,'Spherical-inertia first-order sampled-force path. Endpoint geometry only; no contact response, swept collision qualification, or fluid coupling.',fontsize=9)
    fig.subplots_adjust(bottom=.18,top=.89,hspace=.19,wspace=.08,left=.025,right=.97)
    output.mkdir(parents=True);path=output/'spherical-rigid-motion-example.jpg';fig.savefig(path,dpi=160,pil_kwargs={'quality':85});plt.close(fig)
    with Image.open(path) as im:
        if im.format!='JPEG' or im.size!=(2080,1440):raise ValueError('render dimensions/format')
    result=dict(source_head=receipt['source_head'],native_output_sha256=hashlib.sha256(raw).hexdigest(),jpeg_quality=85,dimensions=[2080,1440],rendered_stored_frame_indices=indices,actual_pose_time_updates=16,display_only_integrations=0,files_sha256={path.name:hashlib.sha256(path.read_bytes()).hexdigest()})
    (output/'render-receipt.json').write_text(json.dumps(result,indent=2)+'\n');return result
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('evidence',type=Path);p.add_argument('output',type=Path);a=p.parse_args();print(json.dumps(render(a.evidence,a.output),indent=2))
