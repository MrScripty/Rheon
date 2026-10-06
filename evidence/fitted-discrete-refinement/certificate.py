"""Exact rational whole-real-path geometry, fifteen-row lift and pressure rank."""
from fractions import Fraction as Q
import json,sys
import temporal as t
import certificate as old
m=t.m

def certify(cell,max_boxes=1024):
 q=list(map(Q,cell['initial_q']));eta=list(map(Q,cell['initial_eta']));alpha=list(map(Q,cell['unknowns'][:6]));h=Q(cell['interval']);pending=[(Q(0),h)];boxes=[];rejections=[]
 while pending:
  lo,hi=pending.pop();mid=(lo+hi)/2
  try:
   D,areas,qualities,_,_=old.rational_geometry(q,eta,alpha,old.Interval(lo,hi));Dc,_,_,_,_=old.rational_geometry(q,eta,alpha,mid)
   chart=old.neumann_bound([[Dc[i][j]for j in t.UNKNOWN]for i in t.LIFT_ROWS],[[D[i][j]for j in t.UNKNOWN]for i in t.LIFT_ROWS])
   pressure=old.neumann_bound([[Dc[i][j]for j in m.PIVOT]for i in m.ROWS],[[D[i][j]for j in m.PIVOT]for i in m.ROWS])
   boxes.append(dict(left=str(lo),right=str(hi),chart_bound=str(chart),pressure_bound=str(pressure),chart_bound_float=float(chart),pressure_bound_float=float(pressure),minimum_microarea=float(min(a.lo for a in areas)),minimum_quality=float(min(a.lo for a in qualities))))
   print('certified box',len(boxes),'width',float(hi-lo),file=sys.stderr,flush=True)
  except ValueError as error:
   rejections.append(dict(left=str(lo),right=str(hi),reason=str(error)));m.require(len(boxes)+len(pending)+2<=max_boxes,'bounded exact certificate subdivisions');pending.extend([(mid,hi),(lo,mid)])
 boxes.sort(key=lambda x:Q(x['left']));m.require(Q(boxes[0]['left'])==0 and Q(boxes[-1]['right'])==h and all(a['right']==b['left']for a,b in zip(boxes,boxes[1:])),'complete real polynomial interval')
 return dict(scope='exact real binary-coefficient polynomial path, no IEEE or mesh-family theorem',interval=cell['interval'],initial_q=cell['initial_q'],initial_eta=cell['initial_eta'],lift_rows=t.LIFT_ROWS.tolist(),unknown_columns=t.UNKNOWN.tolist(),known_columns=t.KNOWN.tolist(),boxes=boxes,rejected_coarse_boxes=rejections,max_boxes=max_boxes,full_fifteen_row_lift_nonsingular=True,full_sixteen_mode_pressure_basis_nonsingular=True,all_microareas_and_quality_positive=True,frozen_symbolic_row_identities=old.row_identities())

if __name__=='__main__':print(json.dumps(certify(json.load(open(sys.argv[1]))),indent=2))
