(use judge)
(use sh)
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

(defmacro enact-sh!
  [sh-command]
  "Run a command only if we aren't in dry-run mode"
  ~(if (true? dry-run)
     (print ,(string/format "Would run %q" sh-command))
     (try
       (do
         ($ ,;sh-command))
       ([err fib]
         (print ,(string/format "failed to run %q" sh-command))))))

(test-macro (enact-sh! (/bin/pkg install rust janet ruby)))
