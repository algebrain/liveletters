(ns liveletters.frontend-app.styles
  (:require [lambdaisland.ornament :as o]))

(def global-styles
  "*,*::before,*::after
{box-sizing:border-box}
:root
{--bg-primary:#18222d;--bg-app:#131b24;--bg-panel:#17202a;--bg-elevated:#1d2834;--bg-soft:#24313e;--bg-hover:#2a3948;--text-primary:#eef4fb;--text-secondary:#95a6b8;--text-tertiary:#6e8092;--accent:#66aee8;--accent-hover:#7ebcf0;--accent-soft:rgba(102,174,232,.14);--border-soft:rgba(255,255,255,.05);--border-strong:rgba(255,255,255,.08);--input-bg:#263342;--input-border:rgba(255,255,255,.06);--input-focus:#66aee8;--modal-overlay:rgba(6,10,14,.68);--danger:#e17c7c;--success:#6bc28f}
html
{font-family:\"Segoe UI\",Inter,Roboto,Helvetica,Arial,sans-serif;background:var(--bg-app);color:var(--text-primary)}
body
{margin:0;min-height:100vh;background:linear-gradient(180deg,#151e27 0%,#18222d 100%)}
#app
{min-height:100vh}
button,input
{font:inherit;color:var(--text-primary)}
.ll-nav
{display:flex;flex-wrap:wrap;gap:10px}
.ll-button
{appearance:none;border:none;border-radius:10px;padding:10px 16px;font-size:.92rem;font-weight:600;letter-spacing:0;cursor:pointer;transition:transform 160ms ease,box-shadow 160ms ease,background-color 160ms ease,border-color 160ms ease,color 160ms ease}
.ll-button:hover
{transform:translateY(-1px)}
.ll-button:disabled
{cursor:not-allowed;opacity:.55;transform:none}
.ll-button--primary
{background:var(--accent);color:#fff;box-shadow:0 10px 24px rgba(58,115,168,.22)}
.ll-button--primary:hover
{background:var(--accent-hover)}
.ll-button--secondary
{background:rgba(255,255,255,.025);color:var(--text-secondary);border:1px solid var(--border-soft)}
.ll-section
{background:linear-gradient(180deg,rgba(31,43,56,.96),rgba(28,39,50,.98));border:1px solid var(--border-soft);border-radius:14px;box-shadow:0 16px 36px rgba(8,12,18,.14)}
.ll-section__title
{margin:0;padding:22px 24px 0;font-size:1.22rem;line-height:1.1;letter-spacing:-.015em;color:var(--text-primary)}
.ll-section__body
{display:grid;gap:16px;padding:18px 24px 22px}
.ll-field
{display:grid;gap:7px}
.ll-field__label
{font-size:.74rem;font-weight:700;text-transform:uppercase;letter-spacing:.09em;color:var(--text-tertiary)}
.ll-input
{width:100%;min-height:46px;padding:11px 14px;border-radius:10px;border:1px solid var(--input-border);background:var(--input-bg);color:var(--text-primary);outline:none;box-shadow:inset 0 1px 0 rgba(255,255,255,.02)}
.ll-input:focus
{border-color:var(--input-focus);box-shadow:0 0 0 3px rgba(102,174,232,.14)}
select.ll-input
{appearance:none;background-image:url(\"data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%236e8092' d='M6 8L1 3h10z'/%3E%3C/svg%3E\");background-repeat:no-repeat;background-position:right 14px center}
select.ll-input option
{background:var(--bg-elevated);color:var(--text-primary)}
.ll-state
{padding:13px 15px;border-radius:10px;font-size:.92rem}
.ll-state--loading
{background:rgba(255,255,255,.06);color:var(--text-secondary)}
.ll-state--empty
{background:rgba(255,255,255,.03);color:var(--text-secondary);border:1px dashed rgba(255,255,255,.05)}
.ll-state--error
{background:rgba(224,85,85,.1);border:1px solid rgba(224,85,85,.2);color:var(--danger)}
.ll-feed
{display:grid;gap:10px;padding:0;margin:0;list-style:none}
.ll-feed__item
{padding:14px 16px;border-radius:10px;background:rgba(31,43,56,.82);border:1px solid rgba(255,255,255,.04);cursor:pointer;box-shadow:0 8px 18px rgba(8,12,18,.08)}
.ll-feed__flag,.ll-post__flag
{display:inline-flex;padding:4px 8px;border-radius:999px;background:rgba(224,85,85,.1);color:var(--danger);font-size:.78rem;font-weight:700}
.ll-thread,.ll-failures,.ll-event-failures
{display:grid;gap:14px;padding:0;margin:0;list-style:none}
.ll-thread__item,.ll-failures__item,.ll-event-failures__item
{padding:16px 18px;border-radius:10px;background:rgba(31,43,56,.82);border:1px solid rgba(255,255,255,.04)}
.ll-sync--healthy
{border-color:rgba(102,174,232,.25)}
.ll-sync--degraded
{border-color:rgba(255,183,77,.3)}
.ll-sync--failed
{border-color:rgba(224,85,85,.25)}
@media (max-width:720px){
  .ll-section__title{padding:18px 18px 0}
  .ll-section__body{padding:16px 18px 18px}
  .ll-button{width:100%}
}")

(defn ensure-style-namespaces-loaded! []
  (require 'liveletters.frontend-app.theme.core
           'liveletters.frontend-app.theme.shell
           'liveletters.frontend-app.theme.nav
           'liveletters.frontend-app.theme.page
           'liveletters.frontend-app.theme.settings))

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
