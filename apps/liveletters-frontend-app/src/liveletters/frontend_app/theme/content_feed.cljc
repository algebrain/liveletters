(ns liveletters.frontend-app.theme.content-feed
  "Стили для ленты постов (Home, Feed, клик на подписку)."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled feed-container :div
  {:padding "20px 24px"
   :max-width "720px"
   :margin "0 auto"})

(o/defstyled feed-header :h2
  {:margin "0 0 16px 0"
   :font-size "1.25rem"
   :font-weight 600
   :color "var(--text-primary)"})

(o/defstyled post-card :article
  {:padding "14px 18px"
   :border-radius "12px"
   :background "var(--bg-tertiary)"
   :border "1px solid rgba(255,255,255,0.06)"
   :margin-bottom "12px"
   :cursor "pointer"
   :transition "background 120ms"}
  [:hover
   {:background "var(--bg-hover)"}])

(o/defstyled post-card-header :div
  {:display "flex"
   :align-items "center"
   :justify-content "space-between"
   :margin-bottom "6px"})

(o/defstyled post-card-author :span
  {:font-size "13px"
   :font-weight 600
   :color "var(--accent)"})

(o/defstyled post-card-time :span
  {:font-size "11px"
   :color "var(--text-secondary)"})

(o/defstyled post-card-body :p
  {:margin "0"
   :font-size "14px"
   :line-height 1.5
   :color "var(--text-primary)"})
