(use ./lib/definers)
(import ../dsl :only [drop-empty-values])
(import ../collector)

(defdoer :ip-interface
  "Create and destroy IP interfaces, with optional properties."
  :name-is "Interface name"
  (ensure
    :finalise-with dsl/drop-empty-values
    ;(protocol-opts)))

(make-fn :ensure :ip-interface)
(make-fn :remove :ip-interface)
