"""Prepare private source copies and test-only Clone overlays before freezing."""
from pathlib import Path
import hashlib,json
P=Path(__file__).resolve().parent
ROOT=P.parents[1]
original=(ROOT/'src/coupled_discrete.rs').read_text()
start=original.index('                // Observe the refused final candidate')
end=original.index('            }\n            // END RHEON REFUSAL DIAGNOSTIC',start)
reference=original[:start]+original[end:]
reference=reference.replace('#![allow(clippy::needless_range_loop)]','#![allow(clippy::needless_range_loop, dead_code)]')
(P/'reference.rs').write_text(reference+'\ninclude!("reference_helpers.rs");\n')
candidate=reference.replace('struct Work {',"struct Work<'a> {\n    research: Option<&'a mut scalar::ChartWorkspace>,",1).replace('    work: Work,',"    work: Work<'a>,",1).replace('pub struct CoupledDiscreteFlow {',"pub struct CoupledDiscreteFlow<'a> {",1).replace('impl CoupledDiscreteFlow {',"impl<'a> CoupledDiscreteFlow<'a> {",1).replace('impl Work {',"impl Work<'_> {",1)
candidate=candidate.replace('        let mut work = Work {','        let mut work = Work {\n            research: None,',1)
old='        let values = linear(block, rhs)?;'
new='''        let values = if let Some(research) = self.research.as_deref_mut() {
            let values = research.solve(&p.d, &p.z)?;
            for i in 0..15 {
                if add(dot(&block[i], &values)?, -rhs[i])?.abs() > LIMIT {
                    return Err(CoupledDiscreteError::LinearFailure);
                }
            }
            values
        } else {
            linear(block, rhs)?
        };'''
if candidate.count(old)!=2:raise ValueError('dependent chart plus unchanged acceleration solves')
candidate=candidate.replace(old,new,1)
candidate=candidate.replace('coordinates(accepted.template, accepted.velocity, &work.r)?','coordinates(accepted.template, accepted.velocity, &work.r, None)?',1)
candidate=candidate.replace('coordinates(accepted.template, accepted.velocity, &self.work.r)?','coordinates(accepted.template, accepted.velocity, &self.work.r, self.work.research.as_deref_mut())?',1)
needle='    r: &[[[f64; V]; 2]; N],\n) -> Result<([f64; 3], [f64; 6]), CoupledDiscreteError>'
if candidate.count(needle)!=1:raise ValueError('one coordinates function')
candidate=candidate.replace(needle,'    r: &[[[f64; V]; 2]; N],\n    research: Option<&mut scalar::ChartWorkspace>,\n) -> Result<([f64; 3], [f64; 6]), CoupledDiscreteError>',1)
needle='    let z = coefficients(u, r)?;\n    Ok((q,'
if candidate.count(needle)!=1:raise ValueError('one existing accepted reconstruction')
candidate=candidate.replace(needle,'    let z = coefficients(u, r)?;\n    if let Some(research) = research {\n        research.set_accepted(z);\n    }\n    Ok((q,',1)
candidate=candidate.replace('use crate::translated_viscous', 'use super::scalar;\nuse crate::translated_viscous',1)
(P/'candidate.rs').write_text(candidate+'\ninclude!("candidate_helpers.rs");\n')
(P/'scalar.rs').write_bytes((ROOT/'evidence/forcing-increment-comparison/scalar.rs').read_bytes())
r=json.loads((ROOT/'evidence/forcing-increment-contract-supplement/selected-capture.json').read_text())
def rust(v):
 if isinstance(v,list):return '['+', '.join(rust(x) for x in v)+']'
 return repr(v) if isinstance(v,float) else str(v)
text=''
for prefix,key in [('INITIAL','initial_publication'),('PREFIX','accepted_publication')]:
 state=r[key]
 for name,value,ty in [('VELOCITY',state['velocity'],'[[f64;3];16]'),('POSITIONS',state['positions'],'[[f64;2];19]'),('MASS',state['mass'],'[f64;16]'),('PRESSURE',state['pressure_coefficients'],'[f64;16]'),('TIME',state['time'],'f64')]:
  text+=f'const {prefix}_{name}: {ty} = {rust(value)};\n'
text+='const ACCEPTED_Z: [f64;22] = '+rust(r['fixed_native_candidate']['start_z'])+';\n'
text+='const H: f64 = '+repr(r['selector']['h'])+';\n'
text+='const BASELINE_TERMINAL_NORM: f64 = '+repr(r['terminal_validation']['rate_norm'])+';\n'
(P/'inputs.rs').write_text(text)
(P/'library-overlay.rs').write_text('#[cfg(test)]\n#[path = "../evidence/forcing-public-call/harness.rs"]\nmod research_public_call;\n')
clones=[]
for path,struct in [('src/fitted_height.rs','FittedHeightWorkspace'),('src/translated_viscous.rs','TranslatedViscousFlow')]:
 before=(ROOT/path).read_text();needle='pub struct '+struct+' {'
 if before.count(needle)!=1:raise ValueError('one clone overlay target')
 after=before.replace(needle,'#[cfg_attr(test, derive(Clone))]\n'+needle,1)
 (P/(struct+'-overlay.rs')).write_text(after)
 clones.append({'path':path,'original_sha256':hashlib.sha256(before.encode()).hexdigest(),'overlay':'evidence/forcing-public-call/'+struct+'-overlay.rs','test_only_change':'derive Clone; no arithmetic/acceptance change'})
(P/'clone-binding.json').write_text(json.dumps(clones,indent=2,sort_keys=True)+'\n')
print('Prepared private source copies; no compilation or numerical execution.')
