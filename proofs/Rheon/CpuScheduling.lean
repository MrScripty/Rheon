import Lean.Elab.Tactic.Omega

/-!
Exact natural-number contracts for Rheon's CPU decomposition and feedback.
These statements do not formalize Rust, Rayon, atomics, machine overflow,
wall-clock progress, numerical error, or a performance optimum.
-/
namespace Rheon.CpuScheduling

/-- The production clamp applied to each proposed linear batch size. -/
def batchSize (proposed : Nat) : Nat := min 16384 (max 64 proposed)

theorem batch_size_bounds (proposed : Nat) :
    64 ≤ batchSize proposed ∧ batchSize proposed ≤ 16384 := by
  unfold batchSize
  omega

/-- Membership in a clipped half-open scratch chunk; all arithmetic is exact. -/
def InChunk (cells grain chunk index : Nat) : Prop :=
  index < cells ∧ chunk * grain ≤ index ∧ index < (chunk + 1) * grain

theorem chunk_covers (cells grain index : Nat) (hg : 0 < grain) (hi : index < cells) :
    InChunk cells grain (index / grain) index := by
  have hdiv := Nat.div_add_mod index grain
  rw [Nat.mul_comm grain (index / grain)] at hdiv
  have hmod := Nat.mod_lt index hg
  unfold InChunk
  rw [Nat.add_mul]
  simp only [Nat.one_mul]
  omega

theorem chunk_unique (cells grain index a b : Nat)
    (ha : InChunk cells grain a index) (hb : InChunk cells grain b index) : a = b := by
  rcases ha with ⟨_, halower, haupper⟩
  rcases hb with ⟨_, hblower, hbupper⟩
  by_cases hab : a < b
  · have h : (a + 1) * grain ≤ b * grain := Nat.mul_le_mul_right grain hab
    omega
  · by_cases hba : b < a
    · have h : (b + 1) * grain ≤ a * grain := Nat.mul_le_mul_right grain hba
      omega
    · omega

/-- The wave rank of the three tile coordinates. -/
def waveRank (x y z : Nat) : Nat := x + y + z

/-- A forward triangular read crossing one tile boundary. -/
def Precedes (ax ay az bx by_ bz : Nat) : Prop :=
  (ax + 1 = bx ∧ ay = by_ ∧ az = bz) ∨
  (ax = bx ∧ ay + 1 = by_ ∧ az = bz) ∨
  (ax = bx ∧ ay = by_ ∧ az + 1 = bz)

theorem predecessor_rank (ax ay az bx by_ bz : Nat)
    (h : Precedes ax ay az bx by_ bz) :
    waveRank ax ay az + 1 = waveRank bx by_ bz := by
  unfold Precedes waveRank at *
  rcases h with h | h | h <;> omega

theorem same_wave_has_no_dependency (ax ay az bx by_ bz : Nat)
    (h : waveRank ax ay az = waveRank bx by_ bz) :
    ¬ Precedes ax ay az bx by_ bz ∧ ¬ Precedes bx by_ bz ax ay az := by
  constructor <;> intro hp
  · have := predecessor_rank ax ay az bx by_ bz hp
    omega
  · have := predecessor_rank bx by_ bz ax ay az hp
    omega

/-- The last rank of a nonempty rectangular tile grid. -/
def lastWave (nx ny nz : Nat) : Nat := nx + ny + nz - 3

theorem tile_wave_in_bounds (nx ny nz x y z : Nat)
    (hx : x < nx) (hy : y < ny) (hz : z < nz) :
    waveRank x y z ≤ lastWave nx ny nz := by
  unfold waveRank lastWave
  omega

theorem reverse_wave_dependency (ax ay az bx by_ bz last : Nat)
    (h : Precedes ax ay az bx by_ bz) (hb : waveRank bx by_ bz ≤ last) :
    last - waveRank bx by_ bz < last - waveRank ax ay az := by
  have := predecessor_rank ax ay az bx by_ bz h
  omega

/-- Exact phase policy; `completed` counts successful, nonoverlapping calls
of one stage with a stable workload, worker count and preferred grain. -/
def trialCandidate (completed : Nat) : Option Nat :=
  if completed % 128 < 8 then some ((completed % 128) % 4) else none

theorem trial_candidate_bounded (completed candidate : Nat)
    (h : trialCandidate completed = some candidate) : candidate < 4 := by
  unfold trialCandidate at h
  split at h
  · simp only [Option.some.injEq] at h
    omega
  · contradiction

theorem trial_first_round (candidate : Nat) (h : candidate < 4) :
    trialCandidate candidate = some candidate := by
  unfold trialCandidate
  have hp : candidate % 128 = candidate := by omega
  have hc : candidate % 4 = candidate := by omega
  simp [hp, hc, show candidate < 8 by omega]

theorem trial_second_round (candidate : Nat) (h : candidate < 4) :
    trialCandidate (candidate + 4) = some candidate := by
  unfold trialCandidate
  have hp : (candidate + 4) % 128 = candidate + 4 := by omega
  have hc : (candidate + 4) % 4 = candidate := by omega
  simp [hp, hc, show candidate + 4 < 8 by omega]

theorem trial_periodic (completed : Nat) :
    trialCandidate (completed + 128) = trialCandidate completed := by
  have h : (completed + 128) % 128 = completed % 128 := by omega
  simp only [trialCandidate, h]

theorem no_trial_after_exploration (completed : Nat) (h : 8 ≤ completed % 128) :
    trialCandidate completed = none := by
  simp [trialCandidate, show ¬ completed % 128 < 8 by omega]

theorem candidate_revisited_within_period (completed candidate : Nat) (h : candidate < 4) :
    ∃ delay : Nat, delay < 128 ∧ trialCandidate (completed + delay) = some candidate := by
  refine ⟨(candidate + 128 - completed % 128) % 128, ?_, ?_⟩
  · omega
  · have hp : (completed + (candidate + 128 - completed % 128) % 128) % 128 = candidate := by
      omega
    unfold trialCandidate
    simp [hp, show candidate < 8 by omega, show candidate % 4 = candidate by omega]

/-- A successful completion increments the phase counter once. Failed,
overlapping, reset or shape-changing operations do not satisfy this premise. -/
def SuccessfulSerialTrace (counter : Nat → Nat) : Prop :=
  ∀ call, counter (call + 1) = counter call + 1

theorem successful_trace_count (counter : Nat → Nat) (h : SuccessfulSerialTrace counter)
    (call : Nat) : counter call = counter 0 + call := by
  induction call with
  | zero => simp
  | succ call ih =>
      have hs := h call
      omega

theorem successful_trace_explores (counter : Nat → Nat) (h : SuccessfulSerialTrace counter)
    (start candidate : Nat) (hc : candidate < 4) :
    ∃ delay : Nat, delay < 128 ∧ trialCandidate (counter (start + delay)) = some candidate := by
  obtain ⟨delay, hd, ht⟩ := candidate_revisited_within_period (counter start) candidate hc
  refine ⟨delay, hd, ?_⟩
  have hs := successful_trace_count counter h start
  have he := successful_trace_count counter h (start + delay)
  have hcounters : counter (start + delay) = counter start + delay := by omega
  rw [hcounters]
  exact ht

end Rheon.CpuScheduling
