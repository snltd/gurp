(use ./lib)
(import ../collector)

(defdoer :etherstub
  "Create and destroy etherstubs."
  :name-is "Name of etherstub")

(make-ensure-fn "etherstub")
(make-remove-fn "etherstub")
