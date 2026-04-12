(ns liveletters.frontend-app.theme.modal
  "Стили для модальных окон (Settings, Add Subscription)."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled modal-overlay :div
  {:position "fixed"
   :top 0
   :left 0
   :right 0
   :bottom 0
   :background "var(--modal-overlay)"
   :display "flex"
   :align-items "center"
   :justify-content "center"
   :z-index 1000})

(o/defstyled modal-content :div
  {:background "linear-gradient(180deg, rgba(22,31,41,0.98), rgba(18,27,36,0.99))"
   :border-radius "14px"
   :width "min(900px, 95vw)"
   :max-height "95vh"
   :overflow-y "auto"
   :box-shadow "0 24px 54px rgba(4,8,14,0.42)"
   :border "1px solid var(--border-soft)"})

(o/defstyled modal-header :div
  {:display "flex"
   :align-items "center"
   :justify-content "space-between"
   :padding "15px 18px"
   :border-bottom "1px solid var(--border-soft)"})

(o/defstyled modal-title :h2
  {:margin 0
   :font-size "1rem"
   :font-weight 600
   :color "var(--text-primary)"})

(o/defstyled modal-close :button
  {:display "flex"
   :align-items "center"
   :justify-content "center"
   :width "30px"
   :height "30px"
   :border-radius "10px"
   :background "transparent"
   :border "none"
   :color "var(--text-tertiary)"
   :cursor "pointer"
   :font-size "18px"
   :line-height 1
   :transition "background 120ms, color 120ms"}
  [:hover
   {:background "rgba(255,255,255,0.05)"
    :color "var(--text-primary)"}])

(o/defstyled modal-body :div
  {:padding "18px"})

;; Add subscription modal specific

(o/defstyled add-sub-input :input
  {:width "100%"
   :min-height "46px"
   :padding "11px 14px"
   :border-radius "10px"
   :border "1px solid var(--input-border)"
   :background "var(--input-bg)"
   :color "var(--text-primary)"
   :font-size "13.5px"
   :outline "none"}
  [:focus
   {:border-color "var(--input-focus)"}])

(o/defstyled add-sub-button :button
  {:padding "10px 22px"
   :border-radius "10px"
   :background "var(--accent)"
   :color "#fff"
   :border "none"
   :font-size "14px"
   :font-weight 600
   :cursor "pointer"
   :transition "background 120ms"}
  [:hover
   {:background "var(--accent-hover)"}])
