(use ./lib/definers)
(import ../collector)

(defdoer :etherstub
  "Create and destroy etherstubs."
  :name-is "Name of etherstub")

(make-fn :ensure :etherstub)
(make-fn :remove :etherstub)
