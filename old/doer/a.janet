#!/usr/bin/env janet

(setdyn :verbose true)
(setdyn :dry-run false)


(comment

(defmacro enact! [obj msg cmd]
  ~(do
     (print (string ,msg ": " ,obj))
     (let [cmd-result (format-command (quote ,cmd))]
       (when (dyn :verbose)
         (print (string "Executing: " cmd-result))))
       # (print (string/format "Executing: %j" (quote ,cmd))))
       # (print (format-command (quote ,cmd))))
    (pp (quote ,cmd))
     (when (not (dyn :dry-run))
       ,cmd)))
      )


(defn immenact! [obj msg cmd]
  (print (string msg ": " obj))
  (when (dyn :verbose)
    (print (string "Executing: "  (string/format "%j" cmd))
    )))

(defn titfuck [& opts]
  (print "cunt")
  (pp opts)

  # (print (type opts))
  # (print (string/join (map |(describe $) opts) " "))
  (print (string/format "eval %s" (first opts)))

  (print "tits"))


(defmacro enact! [obj msg cmd]
  ~(do
    (print (string ,msg ": " ,obj))
    (titfuck ,;cmd)
    )
)

(def path "/tmp/merp")

(enact! path "creating directory" (os/mkdir path))

(pp
  (macex1
    '(enact! path "creating directory" (os/mkdir path))))
