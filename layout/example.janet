(import ../janet_src/lib/gurp)
# (import ./roles/devtools)
(import ./roles/basenode)

(gurp/host "example"
  :roles [
    # "devtools"
    "basenode"
  ])
 
(run-machine-configuration (machine-config))
