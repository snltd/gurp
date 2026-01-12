#
# Interfaces called by the Gurp binary.
# 
(use ./doer-defs)
(use ./defaults)
(import ./formatting)
(import ./doers/directory)
(import ./doers/etherstub)

(defn- doer-lookup
  [doer binding]
  (def lookup (string doer "/" binding))
  (get-in (curenv) [(symbol lookup) :value]))

(defn list-doers
  "Returns a multiline string, pairing doers with their descriptions. Used by
  Gurp's 'doers 'command"
  []
  (def descriptions
    [["directory" directory/description]
     ["etherstub" etherstub/description]])

  (string/join
    (flatten
      (map |(formatting/description-wrapper ;$ 25 80) descriptions))
    "\n"))


(defn- field-width
  "Returns the width of a field which can accomodate the longest value in list"
  [list]
  (if (empty? list)
    0
    (+ 2 (max (splice (map length list))))))

(defn format-properties
  [props]
  (if (empty? props)
    "None"
    (do
      (def field-width-prop-names (field-width (keys props)))
      (def field-width-prop-types (field-width (map |(get $ :types) (values props))))
      (def leader-width (+ field-width-prop-names field-width-prop-types))
      (def leader-format-string
        (string "  %-" field-width-prop-names "s %-" field-width-prop-types "s"))

      (string/join
        (flatten
          (seq [[prop-name prop-vals] :pairs props]
            (def leader
              (string/format leader-format-string prop-name
                             (string/join (prop-vals :types) ", ")))

            (formatting/description-wrapper leader (prop-vals :help) leader-width 80)))
        "\n"))))

(defn help-for
  "Returns a multiline string showing keys supported by the given doer"
  [doer]
  (string
    (formatting/bold-underline doer)
    "\n"
    (string/join (formatting/description-wrapper "" (doer-lookup doer :description) 0 80) "\n")
    "\n"
    "\n"
    (formatting/bold-underline (string doer "/ensure"))
    "\n"
    "name: [string] " (doer-lookup doer :name-is)
    "\n"
    "Mandatory properties"
    "\n"
    (format-properties (doer-lookup doer :mandatory-ensure-props))
    "Optional properties"
    "\n"
    (format-properties (doer-lookup doer :optional-ensure-props))
    "\n"
    "\n"
    (formatting/bold-underline (string doer "/remove"))
    "\n"
    "name: [string] " (doer-lookup doer :name-is)
    "\n"
    "Mandatory properties"
    "\n"
    (format-properties (doer-lookup doer :mandatory-remove-props))
    "Optional properties"
    "\n"
    (format-properties (doer-lookup doer :optional-remove-props))
    "\n"))
