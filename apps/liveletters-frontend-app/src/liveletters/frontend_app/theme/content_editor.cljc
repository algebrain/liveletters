(ns liveletters.frontend-app.theme.content-editor
  "Стили для редактора поста с предпросмотром Markdown."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled editor-container :div
  {:display "flex"
   :flex-direction "column"
   :gap "8px"
   :padding "16px 24px 8px"
   :height "calc(100vh - 46px)"
   :max-height "calc(100vh - 46px)"
   :max-width "1080px"
   :margin "0 auto"})

(o/defstyled editor-topbar :div
  {:display "grid"
   :gap "4px"
   :margin-bottom "0"})

(o/defstyled editor-topbar-copy :div
  {:display "grid"
   :gap "6px"})

(o/defstyled editor-kicker :div
  {:font-size "11px"
   :font-weight 700
   :text-transform "uppercase"
   :letter-spacing "0.1em"
   :color "var(--accent)"})

(o/defstyled editor-header :h2
  {:margin 0
   :font-size "1.42rem"
   :font-weight 600
   :color "var(--text-primary)"})

(o/defstyled editor-subtitle :p
  {:margin 0
   :font-size "13px"
   :line-height 1.5
   :color "var(--text-secondary)"})

(o/defstyled editor-context :div
  {:display "flex"
   :align-items "center"
   :gap "8px"
   :font-size "12px"
   :color "var(--text-tertiary)"})

(o/defstyled editor-context-dot :span
  {:width "8px"
   :height "8px"
   :border-radius "999px"
   :background "var(--accent)"})

(o/defstyled editor-layout :div
  {:display "grid"
   :grid-template-columns "minmax(0, 1.12fr) minmax(300px, 0.88fr)"
   :gap "12px"
   :align-items "stretch"
   :flex "1 1 auto"
   :min-height 0})

(o/defstyled editor-pane :div
  {:display "flex"
   :flex-direction "column"
   :gap "8px"
   :flex "1 1 auto"
   :min-height 0
   :padding "12px 14px"
   :border-radius "10px"
   :background "rgba(28,39,51,0.76)"
   :border "1px solid rgba(255,255,255,0.04)"})

(o/defstyled editor-pane-label :label
  {:font-size "11px"
   :font-weight 700
   :text-transform "uppercase"
   :letter-spacing "0.1em"
   :color "var(--text-tertiary)"})

(o/defstyled editor-textarea :textarea
  {:flex "1 1 auto"
   :width "100%"
   :min-height 0
   :padding 0
   :border-radius 0
   :border "none"
   :background "transparent"
   :color "var(--text-primary)"
   :font-family "Monaco, Menlo, 'Ubuntu Mono', monospace"
   :font-size "14px"
   :line-height 1.7
   :resize "vertical"
   :outline "none"}
  [:focus
   {:border "none"
    :box-shadow "none"}])

(o/defstyled preview-pane :div
  {:display "grid"
   :align-content "start"
   :gap "10px"
   :flex "1 1 auto"
   :min-height 0
   :padding 0
   :border-radius 0
   :background "transparent"
   :border "none"
   :font-size "13.5px"
   :line-height 1.6
   :color "var(--text-primary)"
   :overflow-y "auto"})

(o/defstyled preview-title :h3
  {:margin 0
   :font-size "1.02rem"
   :font-weight 600
   :color "var(--text-primary)"})

(o/defstyled preview-meta :div
  {:font-size "12px"
   :color "var(--text-secondary)"})

(o/defstyled preview-body :div
  {:display "grid"
   :gap "12px"
   :font-size "14px"
   :line-height 1.7
   :color "var(--text-primary)"})

(o/defstyled preview-note :div
  {:padding "12px 14px"
   :border-radius "10px"
   :background "rgba(94,181,247,0.08)"
   :border "1px solid rgba(94,181,247,0.18)"
   :font-size "12px"
   :color "var(--text-secondary)"})

(o/defstyled editor-actions :div
  {:display "flex"
   :justify-content "flex-end"
   :align-items "center"
   :gap "10px"
   :margin-top 0
   :padding "6px 0 2px"})

(o/defstyled secondary-button :button
  {:padding "10px 18px"
   :border-radius "10px"
   :background "transparent"
   :color "var(--text-secondary)"
   :border "1px solid rgba(255,255,255,0.08)"
   :font-size "14px"
   :font-weight 500
   :cursor "pointer"
   :transition "background 120ms, color 120ms"}
  [:hover
   {:background "rgba(255,255,255,0.04)"
    :color "var(--text-primary)"}])

(o/defstyled publish-button :button
  {:padding "10px 22px"
   :border-radius "10px"
   :background "var(--accent)"
   :color "#fff"
   :border "none"
   :font-size "14px"
   :font-weight 600
   :box-shadow "0 10px 20px rgba(94,181,247,0.18)"
   :cursor "pointer"
   :transition "background 120ms, transform 120ms, box-shadow 120ms"}
  [:hover
   {:background "var(--accent-hover)"
    :transform "translateY(-1px)"
    :box-shadow "0 12px 22px rgba(94,181,247,0.22)"}])
