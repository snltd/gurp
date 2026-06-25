(use ./lib)
(import ../collector)

(defdoer :gem
  "Install and uninstall Ruby gems."
  :name-is "Gem name"

  :optional-props-ensure
  {:gem-path {:types [:string]
              :help "Path to gem executable other than /opt/ooce/bin/gem"}
   :source {:types [:string]
            :help "Source other than RubyGems. Can contain tokens and usernames"}
   :version {:types [:string]
             :help "Gem version"}}

  :optional-props-remove
  {:gem-path: {:types [:string]
               :help "Path to gem executable other than /opt/ooce/bin/gem"}
   :version {:types [:string]
             :help "Gem version"}}

  :notes
  ["Tries to minimise the calls to `gem install` by grouping together
           installs with similar parameters"
   "Only version numbers are supported, so `latest` won't work."
   "`gem/remove` takes no options, so removes all versions of the given gem."])

(defensure "gem")
(defremove "gem")
