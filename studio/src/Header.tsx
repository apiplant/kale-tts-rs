export function Header(props: { name: string; repository: string }) {
  const key = `${props.repository}-theme`;
  const root = document.documentElement;
  try {
    const saved = localStorage.getItem(key);
    if (saved === "dark" || saved === "light") root.dataset.theme = saved;
  } catch { /* Use the system theme when storage is unavailable. */ }

  function toggleTheme() {
    const current = root.dataset.theme ??
      (matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark");
    const next = current === "light" ? "dark" : "light";
    root.dataset.theme = next;
    try { localStorage.setItem(key, next); } catch { /* The theme still applies. */ }
  }

  return (
    <header class="site-header">
      <div class="header-inner">
        <h1><a class="brand" href="/">{props.name}<span>-rs</span></a></h1>
        <div class="header-actions">
          <button class="icon-button theme-toggle" type="button" aria-label="Switch theme" title="Switch theme" onClick={toggleTheme}>
            <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4" /><path d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.1 5.1l1.4 1.4M17.5 17.5l1.4 1.4M18.9 5.1l-1.4 1.4M6.5 17.5l-1.4 1.4" /></svg>
          </button>
          <a class="icon-button" href={`https://github.com/apiplant/${props.repository}`} target="_blank" rel="noreferrer noopener"
            aria-label={`${props.repository} on GitHub`} title="Source on GitHub">
            <svg viewBox="0 0 16 16" aria-hidden="true"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.08-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.58.82-2.15-.08-.2-.36-1.01.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27s1.36.09 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.11.16 1.92.08 2.12.51.57.82 1.28.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z" /></svg>
          </a>
        </div>
      </div>
    </header>
  );
}
