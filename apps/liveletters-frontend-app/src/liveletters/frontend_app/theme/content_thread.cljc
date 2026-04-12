(ns liveletters.frontend-app.theme.content-thread
  "Стили для поста с древовидными комментариями."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled thread-container :div
  {:padding "20px 24px"
   :max-width "720px"
   :margin "0 auto"})

(o/defstyled thread-post :article
  {:padding "20px"
   :border-radius "14px"
   :background "var(--bg-tertiary)"
   :border "1px solid rgba(255,255,255,0.06)"
   :margin-bottom "20px"})

(o/defstyled thread-post-title :h2
  {:margin "0 0 8px 0"
   :font-size "1.2rem"
   :color "var(--text-primary)"})

(o/defstyled thread-post-meta :div
  {:font-size "12px"
   :color "var(--text-secondary)"
   :margin-bottom "12px"})

(o/defstyled thread-post-body :p
  {:margin "0"
   :font-size "14px"
   :line-height 1.6
   :color "var(--text-primary)"})

(o/defstyled comment-list :div
  {:display "grid"
   :gap "10px"
   :margin-top "20px"})

(o/defstyled comment-item :div
  {:padding "12px 16px"
   :border-radius "10px"
   :background "var(--bg-secondary)"
   :border-left "3px solid var(--accent)"})

(o/defstyled comment-author :span
  {:font-size "12px"
   :font-weight 600
   :color "var(--accent)"
   :margin-right "8px"})

(o/defstyled comment-body :p
  {:margin "6px 0 0 0"
   :font-size "13px"
   :line-height 1.5
   :color "var(--text-primary)"})

(o/defstyled comment-input-area :div
  {:margin-top "20px"
   :padding "16px"
   :border-radius "12px"
   :background "var(--bg-tertiary)"
   :border "1px solid rgba(255,255,255,0.06)"})

(o/defstyled comment-textarea :textarea
  {:width "100%"
   :min-height "60px"
   :padding "12px"
   :border-radius "10px"
   :border "1px solid var(--input-border)"
   :background "var(--input-bg)"
   :color "var(--text-primary)"
   :font-size "14px"
   :resize "vertical"
   :outline "none"
   :margin-bottom "10px"}
  [:focus
   {:border-color "var(--input-focus)"}])
