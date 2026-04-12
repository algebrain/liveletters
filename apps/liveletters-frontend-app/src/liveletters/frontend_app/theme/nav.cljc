(ns liveletters.frontend-app.theme.nav
  (:require [lambdaisland.ornament :as o]))

(o/defstyled nav-shell :div
  {:display "flex"
   :justify-content "space-between"
   :align-items "center"
   :gap "16px"
   :padding "10px 14px"
   :border-radius "999px"
   :background "rgba(255, 255, 255, 0.06)"
   :border "1px solid rgba(255, 255, 255, 0.08)"}
  [:>.ll-nav-label
   {:font-size "0.8rem"
    :font-weight 700
    :letter-spacing "0.12em"
    :text-transform "uppercase"
    :color "var(--text-secondary)"}])
