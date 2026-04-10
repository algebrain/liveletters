(ns liveletters.frontend-app.theme
  (:require [lambdaisland.ornament :as o]))

(o/defstyled app-shell :main
  {:min-height "100vh"
   :padding "40px 24px 72px"}
  [:>.ll-shell
   {:max-width "1120px"
    :margin "0 auto"
    :display "grid"
    :gap "18px"}])

(o/defstyled nav-shell :div
  {:display "flex"
   :justify-content "space-between"
   :align-items "center"
   :gap "16px"
   :padding "10px 14px"
   :border-radius "999px"
   :background "rgba(255, 250, 242, 0.62)"
   :border "1px solid rgba(101, 77, 53, 0.12)"
   :box-shadow "0 12px 30px rgba(58, 41, 26, 0.08)"}
  [:>.ll-nav-label
   {:font-size "0.8rem"
    :font-weight 700
    :letter-spacing "0.12em"
    :text-transform "uppercase"
    :color "#6a5744"}])

(o/defstyled page-copy :p
  {:max-width "52rem"
   :margin 0
   :font-size "1.08rem"
   :line-height 1.65
   :color "#564738"})

(o/defstyled actions-row :div
  {:display "flex"
   :flex-wrap "wrap"
   :gap "12px"
   :align-items "center"})

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
   :background "linear-gradient(180deg, rgba(255,255,255,0.86) 0%, rgba(248,242,233,0.82) 100%)"
   :border "1px solid rgba(101, 77, 53, 0.10)"}
  [:h3
   {:margin 0
    :font-size "1.15rem"
    :letter-spacing "-0.02em"}])

(o/defstyled settings-column :div
  {:display "grid"
   :gap "14px"})
