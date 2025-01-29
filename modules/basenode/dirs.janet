(import "../../doers/directory")

(directory/is "/export"
  :comment "Make a placeholder for export datasets"
  :owner "root"
  :group "sys"
  :mode "0755")

(directory/is "/export/home"
  :comment "Make a placeholder for export/home datasets"
  :owner "root"
  :group "root"
  :mode "0755")
