import Rheon.SphereFriction
/-! Conditional finite matched ledgers for isolated Coulomb events. No IEEE,
collision-completeness, convergence or persistent-contact claim is made. -/
namespace Rheon.SphereFrictionInterval
noncomputable section
open Rheon.MeshTraction
open Rheon.SphereContact (squared)

def prefixSum (increments : ℕ → ℝ) : ℕ → ℝ
  | 0 => 0
  | n+1 => prefixSum increments n + increments n

theorem prefix_telescopes (state : ℕ → ℝ) (n : ℕ) :
    prefixSum (fun k => state (k+1)-state k) n = state n-state 0 := by
  induction n with
  | zero => simp [prefixSum]
  | succ n ih => rw [prefixSum,ih]; ring

/-- A component of linear, spin or world angular momentum, or actual energy.
Every native/physical increment correspondence is explicitly a premise. -/
theorem matched_prefix_balance (state impulse : ℕ → ℝ) (n : ℕ)
    (matched : ∀ k < n, state (k+1)-state k = impulse k) :
    state n-state 0 = prefixSum impulse n := by
  induction n with
  | zero => simp [prefixSum]
  | succ n ih =>
    have h := ih (fun k hk => matched k (Nat.lt_trans hk (Nat.lt_succ_self n)))
    have last := matched n (Nat.lt_succ_self n)
    rw [prefixSum]
    linarith

theorem matched_midpoint_work_prefix (energy work : ℕ → ℝ) (n : ℕ)
    (coast_and_event : ∀ k < n, energy (k+1)-energy k = work k) :
    energy n-energy 0 = prefixSum work n :=
  matched_prefix_balance energy work n coast_and_event

theorem prefix_nonincrease (energy : ℕ → ℝ) (n : ℕ)
    (coast_and_event : ∀ k < n, energy (k+1) ≤ energy k) :
    energy n ≤ energy 0 := by
  induction n with
  | zero => exact le_refl _
  | succ n ih =>
    have h := ih (fun k hk => coast_and_event k (Nat.lt_trans hk (Nat.lt_succ_self n)))
    exact le_trans (coast_and_event n (Nat.lt_succ_self n)) h

theorem every_accepted_prefix_nonincrease (energy : ℕ → ℝ) (n p : ℕ)
    (hp : p ≤ n) (steps : ∀ k < n, energy (k+1) ≤ energy k) :
    energy p ≤ energy 0 := by
  apply prefix_nonincrease
  intro k hk
  exact steps k (Nat.lt_of_lt_of_le hk hp)

/-- Each exact coast/event split, material bounds and computed-disk bound are
premises. This connects the Coulomb event theorem to every finite prefix. -/
theorem conditional_coulomb_prefix (energy m e vn slip k q : ℕ → ℝ) (n : ℕ)
    (hm : ∀ j < n, 0 ≤ m j) (he0 : ∀ j < n, 0 ≤ e j)
    (he1 : ∀ j < n, e j ≤ 1) (hk : ∀ j < n, 0 ≤ k j)
    (hq : ∀ j < n, 0 ≤ q j) (hcap : ∀ j < n, k j*q j ≤ slip j)
    (matched : ∀ j < n, energy (j+1)-energy j =
      -(m j/2)*(1-(e j)^2)*(vn j)^2-q j*slip j+k j/2*(q j)^2) :
    energy n ≤ energy 0 := by
  apply prefix_nonincrease
  intro j hj
  exact Rheon.SphereFriction.conditional_total_nonincrease
    (energy j) (energy (j+1)) (m j) (e j) (vn j) (slip j) (k j) (q j)
    (hm j hj) (he0 j hj) (he1 j hj) (hk j hj) (hq j hj) (hcap j hj) (matched j hj)

/-- No set-surjectivity claim: membership is equivalent for each rotated
offset under an explicit norm-preserving map. Set equality additionally needs
an inverse/bijection. Collider radius is independent of render mesh rotation. -/
theorem sphere_offset_membership (rotate : Vec → Vec) (offset : Vec) (radius : ℝ)
    (norm_preserving : squared (rotate offset) = squared offset) :
    squared (rotate offset) ≤ radius^2 ↔ squared offset ≤ radius^2 := by
  rw [norm_preserving]

/-- A law-independent certificate: only actual COM velocity enters departure.
The impact's spin need not satisfy a sign predicate for this collider. -/
theorem actual_velocity_departure (c p x actualV : Vec) (radius t : ℝ)
    (hr : 0 < radius) (clearance : radius^2 ≤ squared (sub c p))
    (support : dot (sub c p) (sub x p) ≤ 0)
    (outward : 0 ≤ dot (sub c p) actualV) (ht : 0 ≤ t) :
    radius^2 ≤ squared (sub (add c (Rheon.RigidMotion.scale t actualV)) x) :=
  Rheon.SphereInterval.departure_free_flight c p x actualV radius t hr clearance support outward ht
end
end Rheon.SphereFrictionInterval
