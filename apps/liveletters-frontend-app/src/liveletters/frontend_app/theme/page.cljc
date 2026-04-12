(ns liveletters.frontend-app.theme.page
  (:require [lambdaisland.ornament :as o]))

(o/defstyled page-copy :p
  {:max-width "52rem"
   :margin 0
   :font-size "1.08rem"
   :line-height 1.65
   :color "var(--text-secondary)"})

(o/defstyled actions-row :div
  {:display "flex"
   :flex-wrap "wrap"
   :gap "12px"
   :align-items "center"})
