# package doer

You specify packages by name, so `ooce/editor/helix` rather than
`pkg://sysdef/ooce/editor/helix@25.1-151052.0:20250108t110907Z`. This means you
can't request specific versions. I might change this, but I never pin to
version, and I'm immediately only solving the problems I actually have.

Operating only on name makes the doer run faster, because it knows exactly
what can and cannot be done, so runs `pkg(5)` in the most efficient way
possible `pkg(5)` is rather a slow tool.
