(use ./lib)
(import ../collector)

(defdoer :bridge
  "Create and modify ethernet bridges."
  :name-is "Any valid bridge name"

  :mandatory-props-ensure
  {:protect {:types [:string]
             :help "Protection method"
             :default "stp"}
   :priority {:types [:number]
              :help "Bridge priority. 0 to 61440"
              :default 32768}
   :max-age {:types [:number]
             :help "Maximum age, in seconds, for STP configuration information"
             :default 20}
   :hello-time {:types [:number]
                :help "STP hello time value, in seconds"
                :default 2}
   :forward-delay {:types [:number]
                   :help "STP forward delay time, in seconds. 4 to 30"
                   :default 15}
   :force-protocol {:types [:number]
                    :help "MSTP forced maximum supported protocol"
                    :default 3}}

  :optional-props-ensure
  {:links {:types [:tuple :array]
           :help "Existing links which should be attached to the bridge"}})

(make-ensure-fn "bridge")
(make-remove-fn "bridge")
