(ns liveletters.frontend-app.theme.content-feed
  "Стили для ленты постов (Home, Feed, клик на подписку)."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled feed-container :div
  {:padding "18px 24px 28px"
   :max-width "760px"
   :margin "0 auto"})

(o/defstyled feed-header :h2
  {:margin "0 0 14px 0"
   :font-size "1.18rem"
   :font-weight 600
   :color "var(--text-primary)"})

(o/defstyled post-card :article
  {:padding "12px 15px"
   :border-radius "10px"
   :background "rgba(31,43,56,0.76)"
   :border "1px solid rgba(255,255,255,0.04)"
   :margin-bottom "10px"
   :cursor "pointer"
   :transition "background 120ms, border-color 120ms"}
  [:hover
   {:background "rgba(37,50,64,0.9)"
    :border-color "rgba(255,255,255,0.06)"}])

(o/defstyled post-card-header :div
  {:display "flex"
   :align-items "center"
   :justify-content "space-between"
   :margin-bottom "6px"})

(o/defstyled post-card-author :span
  {:font-size "12.5px"
   :font-weight 600
   :color "var(--accent)"})

(o/defstyled post-card-time :span
  {:font-size "11px"
   :color "var(--text-tertiary)"})

(o/defstyled post-card-body :p
  {:margin "0"
   :font-size "13.5px"
   :line-height 1.52
   :color "var(--text-primary)"})
