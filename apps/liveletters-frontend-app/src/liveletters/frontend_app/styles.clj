(ns liveletters.frontend-app.styles
  (:require [lambdaisland.ornament :as o]))

(def global-styles
  "*
{box-sizing:border-box}
html
{font-family:\"Iowan Old Style\",\"Palatino Linotype\",\"Book Antiqua\",Georgia,serif;background:#f5efe3;color:#1f1b16}
body
{margin:0;min-height:100vh;background:radial-gradient(circle at top,#fff7eb 0%,#f5efe3 48%,#eadfcd 100%)}
#app
{min-height:100vh}
button,input
{font:inherit}
.ll-nav
{display:flex;flex-wrap:wrap;gap:10px}
.ll-button
{appearance:none;border:none;border-radius:999px;padding:12px 18px;font-size:.98rem;font-weight:600;letter-spacing:.01em;cursor:pointer;transition:transform 160ms ease,box-shadow 160ms ease,background-color 160ms ease}
.ll-button:hover
{transform:translateY(-1px)}
.ll-button:disabled
{cursor:not-allowed;opacity:.55;transform:none}
.ll-button--primary
{background:#1f5c4a;color:#fffaf2;box-shadow:0 16px 30px rgba(31,92,74,.22)}
.ll-button--secondary
{background:rgba(255,250,242,.72);color:#284138;border:1px solid rgba(40,65,56,.16)}
.ll-section
{background:rgba(255,250,242,.84);border:1px solid rgba(101,77,53,.14);border-radius:28px;box-shadow:0 28px 60px rgba(58,41,26,.10);backdrop-filter:blur(12px)}
.ll-section__title
{margin:0;padding:28px 30px 0;font-size:clamp(2rem,2.6vw,3rem);line-height:1.05;letter-spacing:-.03em}
.ll-section__body
{display:grid;gap:24px;padding:22px 30px 30px}
.ll-field
{display:grid;gap:8px}
.ll-field__label
{font-size:.82rem;font-weight:700;text-transform:uppercase;letter-spacing:.08em;color:#6a5744}
.ll-input
{width:100%;min-height:48px;padding:12px 14px;border-radius:16px;border:1px solid rgba(79,63,46,.18);background:rgba(255,255,255,.88);color:#231d17;outline:none;box-shadow:inset 0 1px 2px rgba(35,29,23,.04)}
.ll-input:focus
{border-color:#1f5c4a;box-shadow:0 0 0 4px rgba(31,92,74,.12)}
.ll-state
{padding:14px 16px;border-radius:18px;font-size:.96rem}
.ll-state--loading
{background:rgba(232,223,207,.72);color:#4d4033}
.ll-state--empty
{background:rgba(244,239,231,.88);color:#6a5744}
.ll-state--error
{background:rgba(121,29,41,.10);border:1px solid rgba(121,29,41,.16);color:#791d29}
.ll-feed
{display:grid;gap:16px;padding:0;margin:0;list-style:none}
.ll-feed__item
{padding:18px 20px;border-radius:18px;background:rgba(255,255,255,.78);border:1px solid rgba(93,71,48,.12);cursor:pointer}
.ll-feed__flag,.ll-post__flag
{display:inline-flex;padding:4px 8px;border-radius:999px;background:rgba(121,29,41,.10);color:#791d29;font-size:.82rem;font-weight:700}
.ll-thread,.ll-failures,.ll-event-failures
{display:grid;gap:14px;padding:0;margin:0;list-style:none}
.ll-thread__item,.ll-failures__item,.ll-event-failures__item
{padding:16px 18px;border-radius:16px;background:rgba(255,255,255,.72);border:1px solid rgba(93,71,48,.10)}
.ll-sync
{padding:20px 22px;border-radius:20px;background:rgba(255,255,255,.74);border:1px solid rgba(93,71,48,.12)}
.ll-sync--healthy
{border-color:rgba(31,92,74,.20)}
.ll-sync--degraded
{border-color:rgba(145,98,24,.24)}
.ll-sync--failed
{border-color:rgba(121,29,41,.22)}
@media (max-width:720px){
  .ll-section__title{padding:24px 22px 0}
  .ll-section__body{padding:20px 22px 24px}
  .ll-button{width:100%}
}")

(defn ensure-style-namespaces-loaded! []
  (require 'liveletters.frontend-app.theme))

(defn write-styles! []
  (ensure-style-namespaces-loaded!)
  (spit "resources/ornament.css"
        (str global-styles
             "\n"
             (o/defined-styles))))

(defn write-styles-hook
  {:shadow.build/stage :flush}
  [build-state & _args]
  (write-styles!)
  build-state)

(defn -main [& _args]
  (write-styles!))
