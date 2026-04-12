(ns liveletters.frontend-app.theme.shell
  (:require [lambdaisland.ornament :as o]))

(o/defstyled app-shell :main
  {:min-height "100vh"
   :padding "40px 24px 72px"}
  [:>.ll-shell
   {:max-width "1120px"
    :margin "0 auto"
    :display "grid"
    :gap "18px"}])
