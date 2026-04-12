(ns liveletters.frontend-app.theme.content-editor
  "Стили для редактора поста с предпросмотром Markdown."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled editor-container :div
  {:padding "20px 24px"
   :max-width "900px"
   :margin "0 auto"})

(o/defstyled editor-header :h2
  {:margin "0 0 16px 0"
   :font-size "1.25rem"
   :font-weight 600
   :color "var(--text-primary)"})

(o/defstyled editor-layout :div
  {:display "grid"
   :grid-template-columns "1fr 1fr"
   :gap "16px"
   :min-height "400px"})

(o/defstyled editor-pane :div
  {:display "flex"
   :flex-direction "column"})

(o/defstyled editor-pane-label :label
  {:font-size "12px"
   :font-weight 600
   :text-transform "uppercase"
   :letter-spacing "0.08em"
   :color "var(--text-secondary)"
   :margin-bottom "8px"})

(o/defstyled editor-textarea :textarea
  {:flex 1
   :width "100%"
   :padding "14px"
   :border-radius "10px"
   :border "1px solid var(--input-border)"
   :background "var(--input-bg)"
   :color "var(--text-primary)"
   :font-family "Monaco, Menlo, 'Ubuntu Mono', monospace"
   :font-size "13px"
   :line-height 1.6
   :resize "none"
   :outline "none"}
  [:focus
   {:border-color "var(--input-focus)"}])

(o/defstyled preview-pane :div
  {:padding "14px"
   :border-radius "10px"
   :background "var(--bg-tertiary)"
   :border "1px solid rgba(255,255,255,0.06)"
   :font-size "14px"
   :line-height 1.6
   :color "var(--text-primary)"
   :overflow-y "auto"})

(o/defstyled editor-actions :div
  {:display "flex"
   :justify-content "flex-end"
   :gap "10px"
   :margin-top "16px"})

(o/defstyled publish-button :button
  {:padding "10px 24px"
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
