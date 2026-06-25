(use ./lib)
(import ../collector)

(defdoer :bridge
  "Create and modify ethernet bridges."
  :name-is "Any valid bridge name"

  :optional-props-ensure
  {:protect {:types [:string]
             :help "Protection method: defaults to stp"}
   :priority {:types [:number]
              :help "Bridge priority. 0 to 61440"}
   :max-age {:types [:number]
             :help "Maximum age, in seconds, for STP configuration information."}
   :hello-time {:types [:number]
                :help "STP hello time value, in seconds"}
   :forward-delay {:types [:number]
                   :help "STP forward delay time, in seconds. 4 to 30"}
   :force-protocol {:types [:number]
                    :help "MSTP forced maximum supported protocol"}
   :links {:types [:tuple :array]
           :help "Existing links which should be attached to the bridge"}}

  :defaults-ensure
  {:priority 32768
   :protect "stp"
   :forward-delay 15
   :force-protocol 3
   :hello-time 2
   :max-age 20})

(defensure "bridge")
(defremove "bridge")
