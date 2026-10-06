import Mathlib.Data.Real.Basic
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.FieldSimp

/-! Conditional finite exact-real MAC transfer and pressure identities.
No arbitrary geometry/basis integration, Rust/IEEE, solver convergence,
transaction atomicity or moving-surface claim is made. -/
namespace Rheon.ColumnMac
noncomputable section

theorem two_cell_face_mass_partition (a b : ℝ) :
    a/2+(a/2+b/2)+b/2=a+b := by ring

theorem constant_transfer (u : ℝ) : (u+u)/2=u := by ring

theorem fixed_wall_momentum_accounting (m a b : ℝ) :
    m*((a+b)/2)-m*a-m*b=-(m*a/2+m*b/2) := by ring

theorem two_cell_mass_adjoint (m a b v : ℝ) :
    m*((a+b)/2)*v=m*a*(v/2)+m*b*(v/2) := by ring

theorem lift_energy_partition (m a b : ℝ) :
    m*(a^2+b^2)/2-m*((a+b)/2)^2/2=
    m*(a^2+b^2)/4+m*(a-b)^2/8 := by ring

theorem restriction_energy_loss (m v : ℝ) :
    m*v^2/2-m*((v/2)^2+(v/2)^2)/2=m*v^2/4 := by ring

theorem pressure_weight_positive (q m : ℝ) (hq : 0<q) (hm : 0<m) :
    0<q*(q/m) := mul_pos hq (div_pos hq hm)

theorem edge_mass_metric_curvature (q m a b : ℝ) (hm : m≠0) :
    m*(q*(b-a)/m)^2=(q*(q/m))*(b-a)^2 := by
  field_simp
  <;> ring

theorem two_node_pressure_power (a b c p r : ℝ) :
    p*((a+b)*p-a*r)+r*((a+c)*r-a*p)=
    a*(r-p)^2+b*p^2+c*r^2 := by ring

theorem two_node_pressure_power_nonnegative (a b c p r : ℝ)
    (ha : 0≤a) (hb : 0≤b) (hc : 0≤c) :
    0≤a*(r-p)^2+b*p^2+c*r^2 := by positivity

theorem integrated_divergence_gradient_adjoint (a b p r u v : ℝ) :
    p*(-a*u)+r*(a*u-b*v)=u*(a*(r-p))+v*(-b*r) := by ring

theorem weighted_velocity_energy_expansion (m u delta round : ℝ) :
    m*(u+delta+round)^2/2-m*u^2/2=
    m*u*delta+m*delta^2/2+m*round*(u+delta+round/2) := by ring

theorem projection_energy_given_residual_work (before after correction residual rounding bound : ℝ)
    (identity : after-before+correction-residual-rounding=0)
    (hc : 0≤correction) (hw : residual+rounding≤bound) :
    after≤before+bound := by linarith

end
end Rheon.ColumnMac
