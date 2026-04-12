(ns liveletters.frontend-app.theme.settings
  (:require [lambdaisland.ornament :as o]))

(o/defstyled settings-layout :div
  {:display "grid"
   :gap "18px"})

(o/defstyled settings-grid :div
  {:display "grid"
   :grid-template-columns "repeat(auto-fit, minmax(260px, 1fr))"
   :gap "14px"})

(o/defstyled settings-card :section
  {:padding "16px"
   :display "grid"
   :gap "14px"
   :border-radius "12px"
   :background "rgba(30,42,54,0.82)"
   :border "1px solid rgba(255, 255, 255, 0.04)"}
  [:h3
   {:margin 0
    :font-size "1rem"
    :font-weight 600
    :letter-spacing "-0.01em"}])

(o/defstyled settings-column :div
  {:display "grid"
   :gap "12px"})
