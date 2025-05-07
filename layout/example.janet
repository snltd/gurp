(import ./lib/helpers)
(import ./roles/devtools)

# (pp (macex1
# '(helpers/host "example"
#   :vars {
#     :var_a "value a"
#     :var_b ["item_1" "item_2" "item_3"]
#     :var_c 12345
#   }
#   :roles [
#     "devtools"
#   ])

# ))

# (defn machine-config []
#   (helpers/merge-roles
#     (devtools/role)))

(helpers/host "example"
  :vars {
    :var_a "value a"
    :var_b ["item_1" "item_2" "item_3"]
    :var_c 12345
  }
  :roles [
    "devtools"
  ])
 (pp (machine-config))

