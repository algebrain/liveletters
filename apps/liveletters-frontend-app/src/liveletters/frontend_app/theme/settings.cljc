(ns liveletters.frontend-app.theme.settings
  (:require [lambdaisland.ornament :as o]))

(o/defstyled settings-layout :div
  {:display "grid"
   :gap "22px"})

(o/defstyled settings-grid :div
  {:display "grid"
   :grid-template-columns "repeat(auto-fit, minmax(260px, 1fr))"
   :gap "18px"})

(o/defstyled settings-card :section
  {:padding "20px"
   :display "grid"
   :gap "16px"
   :border-radius "22px"
   :background "var(--bg-tertiary)"
   :border "1px solid rgba(255, 255, 255, 0.06)"}
  [:h3
   {:margin 0
    :font-size "1.15rem"
    :letter-spacing "-0.02em"}])

(o/defstyled settings-column :div
  {:display "grid"
   :gap "14px"})
