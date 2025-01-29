# These are settings that you will be able to override from config or 
# command-line at some point
(var verbose true)
(var dry-run false)
(var debug-output false)

(defn say 
  "Print output"
  [& msg]
  (print (string ;msg)))

(defn say-debug 
  "Print output if the debug setting is true"
  [& msg]
  (when (true? debug-output) 
    (say ;msg)))

(defn enact!
  "Run a command only if we aren't in dry-run mode"
  [anon-fn]
  (when (false? dry-run) 
    (anon-fn)))
