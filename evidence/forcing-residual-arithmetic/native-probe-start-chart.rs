// Research-only test overlay. No owner is advanced or candidate published.
use super::*;
struct Case {
    index: usize, q: [f64;3], eta: [f64;6], unknown: [f64;V],
    old: [[f64;3];N], mass: [f64;N], expected_rate: [f64;V],
    expected_norm: f64, h: f64, acceleration: [f64;3],
}
include!("native_inputs.rs");
#[test]
fn captured_refusal_equation_arithmetic() {
    for c in CASES {
        let cap=[[0.,1.],[0.5,1.25],[1.,1.]];
        let geometry=FittedHeightWorkspace::new(FittedHeightGeometry {
            cap:&cap,bottom_x:&BOTTOM,extrusion_width:1.,density:3.,dynamic_viscosity:0.05,
        },Default::default(),|_|false).unwrap();
        let mut work=Work {geometry,r:[[[0.;V];2];N],diagnostic:[Default::default();N],pressure:[0.;P]};
        for i in 0..N {for d in 0..2 {
            let e=work.geometry.velocity_embedding(i,d).unwrap();
            for k in 0..2 {if let Some(j)=e.columns[k] {work.r[i][d][j]=add(work.r[i][d][j],e.weights[k]).unwrap();}}
        }}
        let rebuilt_start=work.point(c.q,c.eta,&c.unknown,0.,&mut |_|false).unwrap();
        let coarse=work.equation(c.q,c.eta,&c.unknown,c.h,16,&c.old,&c.mass,c.acceleration,&mut |_|false).unwrap();
        let fine=work.equation(c.q,c.eta,&c.unknown,c.h,32,&c.old,&c.mass,c.acceleration,&mut |_|false).unwrap();
        assert_eq!(coarse.rate.map(f64::to_bits),c.expected_rate.map(f64::to_bits));
        assert_eq!(norm(&coarse.rate).unwrap().to_bits(),c.expected_norm.to_bits());
        assert!(norm(&coarse.rate).unwrap()>NEWTON);
        let before=TranslatedViscousStamp {id:81,version:0};
        let after=TranslatedViscousStamp {id:81,version:1};
        let quality=qualify(&fine,&coarse,c.h,c.unknown,before,after,c.h,7,50,c.acceleration);
        let (quality_ok,work_fraction,ledger_fraction,gcl,quad)=match quality {
            Ok(r)=>(true,r.residual_work.abs()/r.work_allowance,r.ledger_error.abs()/r.work_allowance,r.gcl_max,r.quadrature_error),
            Err(_)=>(false,0.,0.,0.,0.),
        };
        println!("{{\"case\":{},\"h\":{:?},\"r\":{:?},\"start_z\":{:?},\"rebuilt_start_z\":{:?},\"start_d\":{:?},\"end_z\":{:?},\"end_eta\":{:?},\"start_mass\":{:?},\"end_mass\":{:?},\"end_velocity\":{:?},\"end_force\":{:?},\"end_b\":{:?},\"end_d\":{:?},\"pairs\":{:?},\"plus\":{:?},\"minus\":{:?},\"rate\":{:?},\"direct_rate\":{:?},\"norm\":{:?},\"planar_qualification_pass\":{},\"work_fraction\":{:?},\"ledger_fraction\":{:?},\"gcl\":{:?},\"quadrature\":{:?},\"published\":false}}",c.index,c.h,work.r,coarse.start.z,rebuilt_start.z,rebuilt_start.d,coarse.end.z,coarse.end.eta,coarse.start.mass,coarse.end.mass,coarse.end.u,coarse.end.force,coarse.end.b,coarse.end.d,&coarse.end.pairs[..coarse.end.faces],&coarse.integral.plus[..coarse.end.faces],&coarse.integral.minus[..coarse.end.faces],coarse.rate,coarse.direct_rate,norm(&coarse.rate).unwrap(),quality_ok,work_fraction,ledger_fraction,gcl,quad);
    }
}
