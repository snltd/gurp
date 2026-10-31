(use ./definers)
(import ../collector)

(defdoer :bridge
  "Create and modify ethernet bridges."
  :name-is "Any valid bridge name"
  (ensure
    (property "protect" [:string] :mandatory "Protection method"
              :default "stp")
    (property "priority" [:number] :mandatory "Bridge priority. 0 to 61440"
              :default 32768)
    (property "max-age" [:number] :mandatory
              "Maximum age, in seconds, for STP configuration information"
              :default 20)
    (property "hello-time" [:number] :mandatory
              "STP hello time value, in seconds"
              :default 2)
    (property "forward-delay" [:number] :mandatory
              "STP forward delay time, in seconds. 4 to 30"
              :default 15)
    (property "force-protocol" [:number] :mandatory
              "MSTP forced maximum supported protocol"
              :default 3)
    (property "links" [:tuple] :optional
              "Existing links which should be attached to the bridge")))

(make-fn :ensure "bridge")
(make-fn :remove "bridge")
