(ns liveletters.ui-kit.icons
  "SVG-иконки для UI (Feather Icons, MIT-лицензия).
   Все иконки используют currentColor для окраски через CSS."
  (:require [lambdaisland.ornament :as o]))

(defn icon-back []
  [:svg {:width "20" :height "20" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:polyline {:points "15 18 9 12 15 6"}]])

(defn icon-forward []
  [:svg {:width "20" :height "20" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:polyline {:points "9 18 15 12 9 6"}]])

(defn icon-settings []
  [:svg {:width "20" :height "20" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:circle {:cx "12" :cy "12" :r "3"}]
   [:path {:d "M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"}]])

(defn icon-plus []
  [:svg {:width "20" :height "20" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:line {:x1 "12" :y1 "5" :x2 "12" :y2 "19"}]
   [:line {:x1 "5" :y1 "12" :x2 "19" :y2 "12"}]])

(defn icon-pen []
  [:svg {:width "20" :height "20" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:path {:d "M12 19l7-7 3 3-7 7-3-3z"}]
   [:path {:d "M18 13l-1.5-7.5L2 2l3.5 14.5L13 18l5-5z"}]
   [:path {:d "M2 2l7.586 7.586"}]
   [:circle {:cx "11" :cy "11" :r "2"}]])

(defn icon-home []
  [:svg {:width "18" :height "18" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:path {:d "M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"}]
   [:polyline {:points "9 22 9 12 15 12 15 22"}]])

(defn icon-rss []
  [:svg {:width "18" :height "18" :viewBox "0 0 24 24"
         :fill "none" :stroke "currentColor" :stroke-width "2"
         :stroke-linecap "round" :stroke-linejoin "round"}
   [:path {:d "M4 11a9 9 0 0 1 9 9"}]
   [:path {:d "M4 4a16 16 0 0 1 16 16"}]
   [:circle {:cx "5" :cy "19" :r "1"}]])
