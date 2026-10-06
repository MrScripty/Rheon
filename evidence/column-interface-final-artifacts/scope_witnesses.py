"""Executable remaining scope limits; acceptance here is not qualification."""
import csv
import hashlib
import json
import math
from pathlib import Path
import shutil
import struct
import sys
import tempfile
from PIL import Image

P=Path(__file__).resolve().parent;ROOT=P.parents[1]
sys.path.insert(0,str(ROOT/'tools'))
import verify_column_interface as column
import verify_viscosity as viscosity

def edit(path,change):
    with path.open()as stream:
        reader=csv.DictReader(stream);fields,rows=reader.fieldnames,list(reader)
    change(rows)
    with path.open('w',newline='')as stream:
        writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(rows)

rows=[]
with tempfile.TemporaryDirectory()as temporary:
    demo=Path(temporary)/'column';shutil.copytree(ROOT/'evidence/column-interface/demo',demo)
    target=demo/'jacobi-pcg-v1-activation/frame-02-cells.csv'
    edit(target,lambda data:data[0].__setitem__('pressure','999'))
    result=column.verify(demo)
    rows.append(dict(control='fabricated_later_wet_pressure',status='ACCEPTED_SCOPE_LIMIT',scenarios=len(result['results']),mutated_sha256=hashlib.sha256(target.read_bytes()).hexdigest(),meaning='No independent reconstruction of later held wet pressure solve. Not valid pressure evidence.'))
with tempfile.TemporaryDirectory()as temporary:
    demo=Path(temporary)/'viscosity';shutil.copytree(ROOT/'evidence/viscosity/demo',demo)
    case=demo/'jacobi-pcg-v1-coupled-mu0.125';target=case/'frame-02-faces.csv'
    def reverse(data):
        for row in data:row['velocity']=str(-float(row['velocity']))
    edit(target,reverse);v=viscosity.faces(target,[2,2,1]);image=Image.new('L',(2,2))
    for j in range(2):
        for i in range(2):
            total=struct.unpack('f',struct.pack('f',v[0,i,j,0]+v[0,i+1,j,0]))[0]
            image.putpixel((i,1-j),min(255,max(0,math.floor(255*(.5+.25*total)+.5))))
    image.save(case/'frame-02.png');result=viscosity.verify(demo)
    rows.append(dict(control='reversed_coupled_vector_with_matching_pixels',status='ACCEPTED_SCOPE_LIMIT',mutated_sha256=hashlib.sha256(target.read_bytes()).hexdigest(),meaning='Energy/divergence/pixel checks do not replay coupled vector transitions. Not valid dynamics evidence.'))
print(json.dumps(dict(status='REMAINING_SCOPE_LIMITS_REPRODUCED',controls=rows,closed_by_final_artifact_repair=False,closed_by_newer_coupled_replay=False),indent=2))
