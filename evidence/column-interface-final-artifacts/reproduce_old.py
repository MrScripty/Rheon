"""Preserve the old final-artifact acceptance defect; never mutate real data."""
import csv
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile

P=Path(__file__).resolve().parent
ROOT=P.parents[1]
spec=importlib.util.spec_from_file_location('frozen_column_reader',P/'trials/frozen_verifier.py')
old=importlib.util.module_from_spec(spec);spec.loader.exec_module(old)
rows=[]
for variant in ['nan_final_velocity','nan_final_geometry','corrupt_final_png']:
    with tempfile.TemporaryDirectory() as temporary:
        demo=Path(temporary)/'demo';shutil.copytree(ROOT/'evidence/column-interface/demo',demo)
        case=demo/'jacobi-pcg-v1-activation'
        if variant=='corrupt_final_png':(case/'final.png').write_bytes(b'not a PNG')
        else:
            file='final-faces.csv'if variant=='nan_final_velocity'else'final-geometry.csv'
            field='velocity'if variant=='nan_final_velocity'else'top_fraction'
            path=case/file
            with path.open()as stream:
                reader=csv.DictReader(stream);fields,data=reader.fieldnames,list(reader)
            data[0][field]='nan'
            with path.open('w',newline='')as stream:
                writer=csv.DictWriter(stream,fieldnames=fields);writer.writeheader();writer.writerows(data)
        result=old.verify(demo)
        rows.append(dict(control=variant,old_status='ACCEPTED',scenarios=len(result['results']),scope='Known frozen-reader defect; not valid native evidence'))
print(json.dumps(dict(status='REPRODUCED_OLD_ACCEPTANCE_DEFECT',controls=rows),indent=2))
