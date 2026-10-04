# 14 Guidance as a geometric measurement

Guidance rendering is a measurement layer with a camera model, clipping, sampling and units. It must preserve the spatial information actually simulated. A downstream model may change appearance; Rheon should export reproducible geometric evidence.

## Camera coordinates

For camera coordinates \((X,Y,Z)\) with positive Z forward,

\[
u=f_xX/Z+c_x,\qquad v=f_yY/Z+c_y.
\]

Axial depth \(Z\) differs from radial range \(\sqrt{X^2+Y^2+Z^2}\) off axis. Name the representation, handedness, image origin, clipping and pixel-center convention. Mirroring or shifts may be camera-contract defects rather than fluid errors.

Linearized projection gives

\[
\delta u\approx(f_x/Z)\delta X-(f_xX/Z^2)\delta Z.
\]

The same world-space error moves more pixels near the camera, and depth errors shift off-axis silhouettes. Evaluate actual camera distributions, not just a world-space norm.

## Surface and volume outputs

Liquid can export silhouette, depth and normals in a declared frame. Smoke may integrate extinction. Piecewise constant extinction along a ray gives

\[
T=\exp\left(-\sum_i\sigma_i\Delta s_i\right),
\qquad \alpha=1-T.
\]

Increasing samples without adjusting segment length changes opacity incorrectly. Concentration-to-extinction is a rendering model, not conservation. A diffuse volume has no unique surface; first-hit depth needs a named threshold or accumulated-opacity rule.

Graphics depth buffers may be nonlinear in distance. Exporting raw values as metre depth is incorrect without conversion. Validate planes at known distances and off-axis spheres. Include validity masks rather than ambiguous zero depth.

## Metrics and integration

Silhouette IoU is \(|S\cap T|/|S\cup T|\), with an explicit both-empty case. It can hide a displaced thin filament in a large body. Add boundary-distance percentiles and maximum displacement. Compare depth on mutually valid pixels and report missing surfaces separately. Normals use angular error.

Rest scenes under a fixed camera expose flicker from grid crossings, mesh ambiguity and thresholds. Filtering can reduce flicker while shifting edges, which must remain in the error budget.

Export camera calibration, world scale, dimensions, channel semantics, masks, physical time, state ID and renderer version. Resizing requires updated intrinsics or a stated resampling transform. A depth image without its mapping is ambiguous even when attractive.

The first renderer should be simple and deterministic. Its acceptance is geometric agreement with known primitives and state, not photorealism. Integrated diffusion experiments need separately pinned models and seeds.
