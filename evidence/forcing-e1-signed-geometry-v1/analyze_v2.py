"""Additive metadata repair: normalize NumPy scalars before JSON encoding.

The frozen original analyzer, equations and results are unchanged. Preserve its
first failed serialization attempt; call its original run without altering it.
"""
from pathlib import Path
import importlib.util,json
import numpy as np
P=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('frozen_signed_geometry',P/'analyze.py');original=importlib.util.module_from_spec(spec);spec.loader.exec_module(original)
def scalar(value):
 if isinstance(value,(np.bool_,np.float64)):return value.item()
 raise TypeError('unsupported metadata scalar '+str(type(value)))
def run():return json.loads(json.dumps(original.run(),default=scalar))
if __name__=='__main__':print(json.dumps(run(),indent=2,sort_keys=True))
