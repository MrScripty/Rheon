#!/usr/bin/env python3
"""Bounded exact comparison of existing records ONLY; never launches native work."""
import argparse,json
from pathlib import Path
from check_flat_wall_ritz import compare,require,FILE_CAP
from flat_wall_campaign_guard import write_reserved
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--records',type=Path,required=True);p.add_argument('--source',type=Path,required=True)
    p.add_argument('--head',required=True);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    p.add_argument('--physical-records',action='store_true')
    p.add_argument('--output-byte-cap',type=int,default=FILE_CAP);a=p.parse_args()
    require(1<=a.output_byte_cap<=FILE_CAP,'comparison output quota range')
    repo=Path(__file__).resolve().parents[1];require(not a.output.resolve().is_relative_to(repo),'comparison output must stay outside Git')
    report=compare(a.records,a.source,a.head,a.binary,physical=a.physical_records);text=json.dumps(report,indent=2)+'\n'
    write_reserved(a.output,text.encode(),a.output_byte_cap)
