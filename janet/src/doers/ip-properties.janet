(use ./definers)
(import ../collector)

(defdoer :ip-properties
  "Sets global IP properties, via 'ipadm set-prop'."
  :name-is "Any convenient name: not used internally"
  (ensure ;(protocol-opts))
  (notes "Define `extra_priv_ports` as a comma-separated list."))

(make-fn :ensure :ip-properties)
(make-fn :remove :ip-properties)
