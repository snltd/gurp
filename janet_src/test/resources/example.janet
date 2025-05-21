(use ../../lib/gurp)
(use ./roles/devtools)
(use ./roles/basenode)

(host "example"
  (basenode)
  (devtools)
  )

  (pp
  (machine-config))

# (pp (macex1
# '(host "example"
#   (roles/basenode)
#   (roles/devtools))
#   ))
