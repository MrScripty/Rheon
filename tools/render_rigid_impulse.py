"""JPEG85 of qualified native stored twist at one unchanged mesh pose."""
import argparse
from fractions import Fraction
import hashlib
import json
from pathlib import Path

def render(evidence,output):
    import matplotlib
    matplotlib.use('Agg')
    import matplotlib.pyplot as plt
    from mpl_toolkits.mplot3d.art3d import Poly3DCollection
    from PIL import Image
    root=Path(__file__).resolve().parents[1];evidence=Path(evidence).resolve();output=Path(output).resolve()
    if output.exists() or output.is_relative_to(root):raise ValueError('fresh external render output required')
    receipt=json.loads((evidence/'qualification.json').read_text())
    if receipt.get('qualified') is not True:raise ValueError('qualified native response required')
    name='varying-traction-centroid-counterexample';record=next(r for r in receipt['fixtures'] if r['name']==name)
    raw=(evidence/(name+'.stdout.json')).read_bytes();fixture_raw=(evidence/(name+'.fixture.json')).read_bytes()
    if hashlib.sha256(raw).hexdigest()!=record['stdout_sha256'] or hashlib.sha256(fixture_raw).hexdigest()!=receipt['evidence_sha256'][name+'.fixture.json']:raise ValueError('native/fixture hash mismatch')
    native=json.loads(raw);fixture=json.loads(fixture_raw);points=[[float(Fraction(x)) for x in p] for p in fixture['vertices']]
    fig=plt.figure(figsize=(12,7.5),facecolor='#f6f7fb');ax=fig.add_subplot(121,projection='3d',facecolor='#f6f7fb')
    ax.add_collection3d(Poly3DCollection([points],facecolors='#7babe0',alpha=.35,edgecolors='#365578',linewidths=2))
    # Velocity glyph scale is explicit; geometry is never displaced.
    scale=.3
    for key,color in [('velocity_before_m_s','#617383'),('velocity_after_m_s','#bf4a35')]:
        v=native[key];ax.quiver(0,0,0,*(scale*x for x in v),color=color,linewidth=2.5,arrow_length_ratio=.15)
    ax.set(xlim=(-.2,2.2),ylim=(-.2,1.3),zlim=(0,1.2),xlabel='x (m)',ylabel='y (m)',zlabel='z (m)');ax.view_init(elev=24,azim=-63)
    ax.set_title('One retained pose: COM velocity glyphs\nGray before / red stored after; 0.3 m per (m/s) glyph scale',fontsize=11)
    spin=fig.add_subplot(122,facecolor='#f6f7fb');xs=[0,1,2]
    spin.bar([x-.18 for x in xs],native['angular_before_rad_s'],width=.36,color='#617383',label='before')
    spin.bar([x+.18 for x in xs],native['angular_after_rad_s'],width=.36,color='#bf4a35',label='stored after')
    spin.set(xticks=xs,xticklabels=['world x','world y','world z'],ylabel='Angular velocity (rad/s)',title='Angular response to integrated covariance torque');spin.legend();spin.grid(axis='y',alpha=.2)
    fig.suptitle('Mesh force and torque change owned rigid velocities',fontsize=17,y=.96)
    v0=native['velocity_before_m_s'];v1=native['velocity_after_m_s']
    fig.text(.05,.19,f"Native V (m/s): {v0} → {v1}\nF (N): {native['force_n']}   τ about COM (N m): {native['torque_n_m']}\nMass: 2 kg; world principal moments: (3, 4, 5) kg m²; equivalent force duration: 0.5 s",fontsize=10.5)
    fig.text(.05,.085,f"Native kinetic energy: {native['kinetic_before_j']:.8g} → {native['kinetic_after_j']:.8g} J; signed impulse work: {native['impulse_work_j']:.8g} J\nBody generation {native['generation_before']} → {native['generation_after']}; pose, COM, surface stamp and body time remain fixed.",fontsize=10.5)
    fig.text(.05,.025,'Instantaneous imposed impulse response. No orientation integration, contact resolution, or fluid coupling. Diagnostics are unenclosed.',fontsize=9)
    fig.subplots_adjust(bottom=.32,top=.84,wspace=.3,left=.06,right=.97);output.mkdir(parents=True)
    path=output/'rigid-mesh-impulse-example.jpg';fig.savefig(path,dpi=160,pil_kwargs={'quality':85});plt.close(fig)
    with Image.open(path) as im:
        if im.format!='JPEG' or im.size!=(1920,1200):raise ValueError('render format/dimensions differ')
    manifest=dict(source_head=receipt['source_head'],native_output_sha256=hashlib.sha256(raw).hexdigest(),
                  jpeg_quality=85,dimensions=[1920,1200],physical_time_advances=0,pose_advances=0,
                  velocity_updates=1,files_sha256={path.name:hashlib.sha256(path.read_bytes()).hexdigest()})
    (output/'render-receipt.json').write_text(json.dumps(manifest,indent=2)+'\n');return manifest
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('evidence',type=Path);p.add_argument('output',type=Path);a=p.parse_args();print(json.dumps(render(a.evidence,a.output),indent=2))
