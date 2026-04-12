(ns liveletters.frontend-app.theme.layout
  "Компоненты трёхпанельного layout: top-nav, sidebar, main-content."
  (:require [lambdaisland.ornament :as o]))

(o/defstyled top-nav :nav
  {:grid-row "1"
   :grid-column "1 / span 2"
   :display "flex"
   :align-items "center"
   :justify-content "space-between"
   :padding "0 14px 0 10px"
   :height "46px"
   :background "rgba(19,27,36,0.92)"
   :backdrop-filter "blur(14px)"
   :border-bottom "1px solid var(--border-soft)"})

(o/defstyled sidebar :aside
  {:grid-row "2"
   :grid-column "1"
   :width "286px"
   :background "linear-gradient(180deg, rgba(21,29,38,0.96), rgba(24,34,45,0.98))"
   :border-right "1px solid var(--border-soft)"
   :overflow-y "auto"
   :display "flex"
   :flex-direction "column"})

(o/defstyled main-content :main
  {:grid-row "2"
   :grid-column "2"
   :overflow-y "auto"
   :background "linear-gradient(180deg, rgba(26,36,47,0.98), rgba(25,35,46,1))"
   :padding "0"})
