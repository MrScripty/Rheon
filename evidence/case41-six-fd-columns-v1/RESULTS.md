# Case41 stopped at memory preflight

The actual frozen ELF passed all three exact scalar/layout/affine tests. The independent scalar oracle verified 23 assertion groups, 36 rational probes and all six affine probes. The static memory audit then refused because its reached-frame graph could not match `Work::equation` between candidate FD observation and reference capture routes. This is a qualification failure, not evidence of a measured cap overrun or a numerical equation refusal. No FD memory bound is established.

Per the explicit stop-on-preflight-failure instruction, the seven-equation capture was not invoked. Zero candidate equations, corrections, accepted owners or owner advances. Full scaled Jacobian/range/complement results remain unavailable. The source and all actual failed audit logs are retained, and no source repair or second preflight/capture is attempted in this experiment. Existing scaled pressure analysis and every older result/gate remain unchanged.

Experiment source: f2d2183346fcab7f6a825b831e3f15c194ed4c96; tree da4ec51815425d758dbb24578f776c2c8d1ca962. Compiled source: c238111d8c8d159c157e489294a25998aaf36516; tree c87e9c658cad27df708a34aa63bccf9868095598. Actual ELF SHA-256: 48f58e62577eef569e942d3e8e7139701613091076a736c015bee5256691ba72.
