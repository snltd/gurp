(use ../../lib/gurp)
(import ./roles/devtools)
(import ./roles/basenode)

(host "example"
  :roles [
    "basenode"
    "devtools"
  ])

  # (pp
  # (machine-config))
