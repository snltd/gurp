(import ../../lib/gurp)
(import ./roles/devtools)
(import ./roles/basenode)

(gurp/host "example"
  :roles [
    "basenode"
    "devtools"
  ])

  # (pp
  # (machine-config))
