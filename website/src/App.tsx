import { For, Show, createSignal } from "solid-js";
import { project, type PreviewKind } from "./project";
import "./app.css";

export type Page = "home" | "docs" | "examples" | "missing";
const nav = [
  { label: "Overview", page: "home", href: "/" },
  { label: "Examples", page: "examples", href: "/examples" },
  { label: "Docs", page: "docs", href: "/docs" },
] as const;

export function App(props: { page: Page }) {
  const [light, setLight] = createSignal(document.documentElement.dataset.theme === "light" ||
    (!document.documentElement.dataset.theme && matchMedia("(prefers-color-scheme: light)").matches));
  const toggleTheme = () => {
    const next = !light();
    setLight(next);
    document.documentElement.dataset.theme = next ? "light" : "dark";
    try { localStorage.setItem("theme", next ? "light" : "dark"); } catch { /* Theme still applies. */ }
  };

  return (
    <>
      <header class="site-header">
        <div class="shell header-inner">
          <a href="/" class="brand">{project.name.slice(0, -3)}<span>-rs</span></a>
          <nav aria-label="Main navigation">
            <For each={nav}>{(item) => (
              <a href={item.href} aria-current={props.page === item.page ? "page" : undefined}>{item.label}</a>
            )}</For>
          </nav>
          <button type="button" class="theme-toggle" aria-label={light() ? "Use dark theme" : "Use light theme"} onClick={toggleTheme}>
            <span aria-hidden="true">{light() ? "☾" : "☀"}</span>
          </button>
        </div>
      </header>

      <main class="shell">
        <Show when={props.page === "home"}><Home /></Show>
        <Show when={props.page === "examples"}><Examples /></Show>
        <Show when={props.page === "docs"}><Docs /></Show>
        <Show when={props.page === "missing"}>
          <section class="page-head">
            <h1>Page not found</h1>
            <a class="button" href="/">Back to {project.name}</a>
          </section>
        </Show>
      </main>

      <footer class="shell">
        <span>{project.name}</span>
        <a href={project.modelUrl}>{project.modelLabel}</a>
        <a href="/docs#license">License</a>
      </footer>
    </>
  );
}

function Home() {
  return (
    <>
      <section class="hero">
        <div>
          <h1>{project.headline}</h1>
          <p class="lead">{project.description}</p>
          <div class="actions">
            <a class="button primary" href="/docs#install">Get started</a>
            <a class="button" href="/examples">See examples</a>
          </div>
        </div>
        <figure class="showcase">
          <figcaption><span>{project.exampleTitle}</span><a href="/examples">Examples →</a></figcaption>
          <Preview kind={project.kind} />
        </figure>
      </section>

      <section class="capabilities" aria-label="Features">
        <For each={project.features}>{(feature) => (
          <article><h2>{feature.title}</h2><p>{feature.text}</p></article>
        )}</For>
      </section>

      <section class="home-start">
        <div><h2>Run it locally</h2><p>Start the server, then open the browser tool at <code>127.0.0.1:8080</code>.</p></div>
        <pre><code>{project.serve}</code></pre>
        <a href="/docs#server">Local tool and API →</a>
      </section>
    </>
  );
}

function Preview(props: { kind: PreviewKind; expanded?: boolean }) {
  return (
    <div class={`preview preview-${props.kind}`}>
      <Show when={props.kind === "speech"}>
        <div class="speech-sample">
          <p class="quote">“Hello there. This voice was generated locally with Kale Nano.”</p>
          <audio controls preload="metadata" src="/media/nano-s057b.wav" aria-label="Kale Nano speech with the s057b voice" />
          <small>Nano · s057b · 24 kHz</small>
        </div>
      </Show>
      <Show when={props.kind === "document"}>
        <div class="document-sample">
          <figure><img src="/media/page-1.png" alt="Example PDF, first page reading Alpha page" width="400" height="180" /><figcaption>PDF page</figcaption></figure>
          <div class="document-output"><code>Alpha page</code><small>Extracted Markdown</small></div>
        </div>
        <Show when={props.expanded}>
          <div class="document-sample second-page">
            <figure><img src="/media/page-2.png" alt="Rotated second PDF page reading Beta page" width="160" height="360" /><figcaption>Rotated page</figcaption></figure>
            <div class="document-output"><code>Beta page</code><small>Extracted Markdown</small></div>
          </div>
        </Show>
      </Show>
      <Show when={props.kind === "emoji"}>
        <div class="emoji-sample">
          <img src="/media/sleepy-cats.png" alt="Four generated pixel-art cats wearing tiny crowns" />
          <small>“A sleepy cat with a tiny crown” · seed 7</small>
        </div>
      </Show>
    </div>
  );
}

function Examples() {
  return (
    <>
      <section class="page-head"><h1>{project.exampleHeading}</h1><p class="lead">{project.exampleDescription}</p></section>
      <figure class="showcase examples">
        <figcaption>{project.exampleTitle}</figcaption>
        <Preview kind={project.kind} expanded />
      </figure>
      <div class="actions downloads">
        <For each={project.downloads}>{(item) => <a class="button" href={item.href} download>{item.label}</a>}</For>
      </div>
      <section class="example-command">
        <h2>Reproduce the example</h2>
        <pre><code>{project.exampleCommand}</code></pre>
      </section>
    </>
  );
}

function Docs() {
  return (
    <div class="docs-layout">
      <aside>
        <nav aria-label="Documentation sections">
          <For each={project.docs}>{(section) => <a href={`#${section.id}`}>{section.title}</a>}</For>
        </nav>
      </aside>
      <article class="docs">
        <div class="page-head"><h1>Documentation</h1><p class="lead">{project.docsIntro}</p></div>
        <For each={project.docs}>{(section) => (
          <section id={section.id}>
            <h2>{section.title}</h2>
            <p>{section.text}</p>
            <Show when={section.code}><pre><code>{section.code}</code></pre></Show>
            <Show when={section.routes?.length}>
              <dl class="routes"><For each={section.routes}>{(route) => (<div><dt><code>{route[0]}</code></dt><dd>{route[1]}</dd></div>)}</For></dl>
            </Show>
            <Show when={section.links?.length}>
              <ul><For each={section.links}>{(link) => <li><a href={link.href}>{link.label}</a></li>}</For></ul>
            </Show>
          </section>
        )}</For>
      </article>
    </div>
  );
}
