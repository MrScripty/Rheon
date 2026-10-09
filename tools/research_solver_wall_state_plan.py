"""Read-only actual-state audit and exact, no-allocation physics preflight.
No viscosity/pressure solve, timestep, analytic velocity acquisition or load input.
Topology enumeration uses a bounded integer occupancy buffer and streams faces.
All resulting layouts are proposals, never admitted production operators.
"""
from array import array
from fractions import Fraction as Q
from itertools import product
from pathlib import Path
import argparse, hashlib, json, subprocess
import sys
sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'research'))
from check_proof_audit import require_external_output

ROOT=Path(__file__).resolve().parents[1]
CAP=16_000_000
BASE='4a268efd32d6429000ef73d1dd9ccd606ecebe1c'
def sha(p):return hashlib.sha256(Path(p).read_bytes()).hexdigest()

def enumerate_collar():
    N=24;empty=-1;vox=array('i',[empty])*(N**3);leaves=[]
    def insert(origin,width,level):
        idx=len(leaves);leaves.append((origin,width,level))
        for delta in product(range(width),repeat=3):
            p=[a+b for a,b in zip(origin,delta)];k=p[0]+N*(p[1]+N*p[2]);assert vox[k]==empty;vox[k]=idx
    for c in product(range(12),repeat=3):
        if all(2<=a<10 for a in c):continue
        insert(tuple(2*a for a in c),2,0)
    for c in product(range(4,20),repeat=3):
        if all(8<=a<16 for a in c):continue
        insert(c,1,1)
    def at(p):
        return vox[p[0]+N*(p[1]+N*p[2])] if all(0<=a<N for a in p) else empty
    counts={'cells':len(leaves),'coarse_cells':sum(c[2]==0 for c in leaves),'fine_cells':sum(c[2]==1 for c in leaves),'open_faces':0,'outer_faces':0,'solid_faces':0,'coarse_interface_faces':0,'subfaces':0,'incidences':0}
    # Count each interface once. Coarse–coarse/boundary faces are coalesced;
    # coarse–fine faces remain physical fine subfaces, not fictitious full faces.
    for n in range(3):
        tang=[k for k in range(3) if k!=n]
        for plane in range(N+1):
            for i,j in product(range(N),repeat=2):
                p=[0]*3;p[n]=plane;p[tang[0]]=i;p[tang[1]]=j;q=p.copy();q[n]-=1;a,b=at(q),at(p)
                if a==b:continue
                if a<0 and b<0:continue
                ids=[k for k in (a,b) if k>=0];width=min(leaves[k][1] for k in ids)
                if i%width or j%width:continue
                if a>=0 and b>=0:
                    counts['open_faces']+=1;counts['incidences']+=2
                    if leaves[a][2]!=leaves[b][2]:
                        counts['subfaces']+=1
                        if i%2==0 and j%2==0:counts['coarse_interface_faces']+=1
                else:
                    counts['incidences']+=1
                    counts['outer_faces' if plane in (0,N) else 'solid_faces']+=1
    counts['total_faces']=counts['open_faces']+counts['outer_faces']+counts['solid_faces'];counts['enumerator_integer_buffer_bytes']=len(vox)*vox.itemsize
    assert counts==dict(cells=4800,coarse_cells=1216,fine_cells=3584,open_faces=14352,outer_faces=864,solid_faces=384,coarse_interface_faces=384,subfaces=1536,incidences=29952,total_faces=15600,enumerator_integer_buffer_bytes=55296)
    return counts

def resource(counts,layout):
    C,F,T,I=[counts[k] for k in ('cells','open_faces','total_faces','incidences')]
    outer_ghost_cells=20**3-16**3;outer_ghost_faces=3*21*20**2-3*17*16**2
    solid_ghost_reserve=8**3+3*9*8**2;targets=outer_ghost_cells+outer_ghost_faces+solid_ghost_reserve
    shared={'cell_records':layout['cell']*C,'face_records':layout['face']*T,'eight_f64_face_vectors':64*F,'seven_f64_cell_vectors':56*C,'two_f32_tracer_vectors':8*C,'cell_face_CSR':4*(C+1)+4*I,'coexisting_N12_baseline_operators_and_filled_box_owner':5_454_440,'interface_child_map':20*384,'four_flux_register_values_per_interface':32*384,'fine_parent_children_table':32*448,'two_face_vectors_for_goal_duals':16*F,'bounded_128_row_tile':128*layout['tile_row'],'wall_observation_rows':96*864,'bounded_provenance_reader':65536,'stream_buffer':8192}
    plans={}
    for method,row in [('eight_donor_proposal',layout['ghost_transfer']),('twenty_seven_donor_proposal',layout['quadratic_transfer'])]:
        items=shared|{'all_ghost_transfer_records_and_values':targets*(row+8)};total=sum(items.values());assert total<=CAP
        plans[method]={'line_items':items,'total_managed_bytes':total,'remaining_to_cap':CAP-total,'transfer_qualified':False,'operator_qualified':False}
    N=24;allfaces=3*(N+1)*N*N;cells=N**3
    large={'original_aligned_operator_and_lifts':43_321_344,'geometry':16*cells+8*allfaces+480,'existing_static_pressure':56*cells+8*allfaces+8,'proposed_two_full_f64_state_vectors':16*allfaces,'proposed_eight_active_f64_viscous_CG_vectors':64*38_016,'wall_observation_rows':96*864,'bounded_provenance_reader':65536,'stream_buffer':8192}
    return {'ghost_outer_cells':outer_ghost_cells,'ghost_outer_faces':outer_ghost_faces,'solid_ghost_targets_reserved':solid_ghost_reserve,'total_ghost_targets_reserved':targets,'composite_plans':plans,'legacy_uniform_N24_hypothetical_only':{'line_items':large,'declared_payload_bytes':sum(large.values()),'hypothetical_cap':50_000_000,'remaining_to_hypothetical_cap':50_000_000-sum(large.values()),'current_cap':CAP,'run_performed':False,'larger_cap_justified_now':False}}

def affine_obstruction():
    area=Q(1,64);distance=Q(3,16);mass=area*distance;offset=Q(1,16);H=Q(1,4)
    false=offset/distance;remote=offset/(2*H*distance);gap=mass*remote
    assert (false,remote,gap)==(Q(1,3),Q(2,3),Q(1,512))
    return {'normal_distance':str(distance),'tangential_offset':str(offset),'face_area':str(area),'declared_mass':str(mass),'naive_affine_normal_gradient_magnitude':str(false),'reconstructed_gradient_remote_coefficient_magnitude':str(remote),'mass_adjoint_remote_work_coefficient_magnitude':str(gap),'corrected_transpose_is_geometric_incidence':False,'diagnostic_only_no_pressure_solve':True}

def main():
    p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();out=a.output.resolve();out=require_external_output(out,ROOT);out.mkdir(parents=True,exist_ok=False)
    binary=a.binary.resolve();record=out/'state-audit.json';subprocess.run([str(binary),str(record)],check=True);raw=json.loads(record.read_text());assert raw['cap']==CAP
    assert raw['proposed_layout']==dict(cell=24,face=64,ghost_transfer=96,quadratic_transfer=328,tile_row=1040)
    for s in raw['states']:
        n=s['N'];faces=3*(n+1)*n*n;cells=n**3
        assert s['simulation_bytes']==8*faces+56*cells and s['free_slip_viscosity_bytes']==8*faces
        assert s['face_values']==faces and s['queried_positions']==sum(12*k*k for k in (8,16,32))
        assert s['generation']==s['time']==s['max_interpolated_speed']==0
        assert s['new_independent_information']==s['obstacle_velocity_state']==False and s['read_only']==True
    counts=enumerate_collar()
    report={'schema':'actual-solver-wall-state-plan-v1','parent':BASE,'binary_sha256':sha(binary),'source_sha256':{str(f.relative_to(ROOT)):sha(f) for f in (ROOT/'examples/research_solver_wall_state_audit.rs',Path(__file__))},'actual_record_sha256':sha(record),'actual_state_audit':raw,'topology':counts,'resource_proposals':resource(counts,raw['proposed_layout']),'affine_pressure_obstruction':affine_obstruction(),'velocity_oracle_inputs':False,'solve_step_pressure_coupling_publication':False,'physical_state_error_bound_available':False}
    (out/'plan-diagnostics.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'actual_state_bytes':[s['simulation_bytes']+s['free_slip_viscosity_bytes'] for s in raw['states']],'composite_payloads':{k:v['total_managed_bytes'] for k,v in report['resource_proposals']['composite_plans'].items()},'larger_cap_justified_now':False,'mass_adjoint_pairing_gap':str(Q(1,512))}))
if __name__=='__main__':main()
