(use ../src/facts)
(use judge)

(deftest uname-x->struct
  (test (uname-x->struct
          `System = SunOS
Node = serv-build
Release = 5.11
KernelID = omnios-r151056-1acbca4f5bd
Machine = i86pc
BusType = <unknown>
Serial = <unknown>
Users = <unknown>
OEM# = 0
Origin# = 1
NumCPU = 4`)
    {:bustype "<unknown>"
     :kernelid "omnios-r151056-1acbca4f5bd"
     :machine "i86pc"
     :node "serv-build"
     :numcpu 4
     :oem 0
     :origin 1
     :release 5.11
     :serial "<unknown>"
     :system "SunOS"
     :users "<unknown>"}))

(deftest unknown-fact
  (test-error (fact "wat") "unknown fact: wat"))

(deftest ip-no-loopback
  (test (ip-no-loopback `ADDROBJ           TYPE     STATE        ADDR
lo0/v4            static   ok           127.0.0.1/8
build_net0/_a     from-gz  ok           192.168.1.23/24
lo0/v6            static   ok           ::1/128`)
        {"build_net0/_a" {:addr "192.168.1.23/24"
                          :state "ok"
                          :type "from-gz"}})

  (test (ip-no-loopback `IFNAME     CLASS     STATE    CURRENT      PERSISTENT
lo0        VIRTUAL   ok       -m-v------46 ---
build_net0 IP        ok       bm-------Z4- -4-`)
        {"build_net0" {:class "IP"
                       :current "bm-------Z4-"
                       :persistent "-4-"
                       :state "ok"}}))
