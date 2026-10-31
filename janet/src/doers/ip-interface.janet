(use ./definers)
(import ../collector)

(defdoer :ip-interface
  "Create and destroy IP interfaces, with optional properties."
  :name-is "Interface name"
  (ensure ;(protocol-opts)))

(make-fn :ensure :ip-interface)
(make-fn :remove :ip-interface)
