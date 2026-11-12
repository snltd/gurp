(def interfaces ["mh0" "mh1" "mh2"])

(each iface interfaces
  (ip-interface/ensure iface))

(ipmp/ensure "ipmp0" :interfaces interfaces)
