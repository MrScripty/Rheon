# E A worked matrix laboratory

Small exact systems make useful implementation fixtures because every entry has a meaning. They reveal sign errors, gauge mistakes and false conservation claims without a large visual scene.

## Three cells on a line

Orient two faces from cell 0 to 1 and from 1 to 2. With unit weights,

\[
B=\begin{bmatrix}-1&0\\1&-1\\0&1\end{bmatrix},
\qquad
A=BB^T=\begin{bmatrix}1&-1&0\\-1&2&-1\\0&-1&1\end{bmatrix}.
\]

Take face velocity \((2,-1)\) and time step \(1/2\). Incidence is \((-2,3,-1)\), so right-hand side is \((-4,6,-2)\). It sums to zero, as required. Fixing \(p_0=0\) leaves

\[
\begin{bmatrix}2&-1\\-1&1\end{bmatrix}
\begin{bmatrix}p_1\\p_2\end{bmatrix}
=\begin{bmatrix}6\\-2\end{bmatrix}.
\]

The second equation gives \(p_2=p_1-2\), hence \(p_1=4\), \(p_2=2\). Gradient is \((4,-2)\). Subtracting half of it gives zero velocity. A closed one-dimensional chain has no nonzero divergence-free internal flux; the result follows from topology, not excessive numerical damping.

Now stop at \(p=(0,3.9,2)\). Gradient is \((3.9,-1.9)\), corrected velocity is \((0.05,-0.05)\), and incidence is \((-0.05,0.10,-0.05)\). Residual is \((-0.10,0.20,-0.10)\), exactly twice the corrected incidence. This fixture checks the time-step factor and the sign of physical divergence.

## A cycle retains circulation

A three-cell cycle has faces 0 to 1, 1 to 2 and 2 to 0:

\[
B=\begin{bmatrix}-1&0&1\\1&-1&0\\0&1&-1\end{bmatrix}.
\]

The constant face vector \((c,c,c)\) has zero incidence. With unit metric, projecting velocity \((3,0,0)\) onto this nullspace yields \((1,1,1)\). Its circulation survives while compressive content is removed. Initial squared norm is 9; final squared norm is 3. A test expecting zero velocity in every closed domain would be wrong because it confuses a chain with a cycle.

This graph is an algebraic teaching fixture, not a spatially resolved three-cell physical vortex. Its role is to make the nullspace distinction visible. A production mesh must separately establish which cycles and coefficients correspond to its geometry.

## Deriving energy reduction

Let \(W\) be positive definite, \(u^+=u-\Delta tWB^Tp\), and \(Bu^+=0\). Then the removed vector \(v=\Delta tWB^Tp\) is orthogonal to \(u^+\) in the weighted inner product:

\[
(u^+)^TW^{-1}v
=\Delta t(Bu^+)^Tp=0.
\]

Since \(u=u^++v\),

\[
\|u\|_{W^{-1}}^2
=\|u^+\|_{W^{-1}}^2+\|v\|_{W^{-1}}^2.
\]

This establishes exact norm nonincrease for the stated closed compatible projection. If some weights vanish, \(W^{-1}\) is undefined and the argument needs restriction to active faces or a different metric. Moving boundaries and prescribed flux add work; approximate solves leave a nonzero cross term. The result is derived here but is not one of the current checked Lean theorems.

For an approximate pressure, the cross term is tied to residual. In the unweighted case,

\[
(u^+)^T(\Delta t B^Tp)
=\Delta t(Bu^+)^Tp
=\Delta t^2r^Tp.
\]

Thus solver error can affect energy even when the algebraic operator is correct. A small residual in one norm does not bound this term without also controlling pressure in the dual norm.

## Residual and solution error

On a gauge-fixed SPD system, exact solution \(p_*\) and approximate \(p\) satisfy \(A(p_*-p)=r\). Therefore

\[
\|p_*-p\|_2\leq
\lambda_{\min}(A)^{-1}\|r\|_2.
\]

The smallest eigenvalue depends on geometry, gauge and scaling. A universal pressure-error claim from a residual threshold is invalid without this spectral factor. Velocity error can be better related through the energy norm:

\[
\|B^T(p_*-p)\|_2^2
=(p_*-p)^TA(p_*-p).
\]

These are exact linear-system relations. Floating arithmetic, variable weights and physical metrics need corresponding versions. Their practical lesson is to choose diagnostics from the consumer: divergence tolerance, velocity error and pressure accuracy are related but not identical acceptance criteria.

## A conservation fixture with fractions

Take scalar amounts \((1/3,1/2,1/6)\), cyclic face fluxes \((2/7,-1/5,1/9)\), and time step \(1/10\). Each cell adds incoming minus outgoing amount. The result is \((199/630,96/175,61/450)\), whose sum is exactly one. The companion executes this with rational arithmetic.

This fixture isolates cancellation. It does not prove the selected flux is physically accurate or positivity-preserving for every input. A complete transport validation adds the required flux law, time restriction, geometry and boundary terms. Conservation is a structural property; accuracy depends on how the flux approximates the intended equation.
