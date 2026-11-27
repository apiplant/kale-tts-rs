import { Header } from "./Header";
import { For, Show, createSignal, onCleanup, onSettled } from "solid-js";

type Voice = { id: string; name: string; accent: string };
type Info = { model: string; voices: Voice[] };

export function App() {
  const [text, setText] = createSignal("");
  const [voice, setVoice] = createSignal("");
  const [voices, setVoices] = createSignal<Voice[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal("");
  const [audioUrl, setAudioUrl] = createSignal("");
  const controller = new AbortController();

  async function loadVoices() {
    setLoading(true);
    setError("");
    try {
      const response = await fetch("/voices", { signal: controller.signal });
      if (!response.ok) throw new Error();
      const data = await response.json() as Info;
      if (!data.voices.length) throw new Error();
      setVoice(data.voices[0].id);
      setVoices(data.voices);
    } catch {
      if (!controller.signal.aborted) setError("Could not load voices.");
    } finally {
      setLoading(false);
    }
  }

  onSettled(() => { void loadVoices(); });
  onCleanup(() => {
    controller.abort();
    if (audioUrl()) URL.revokeObjectURL(audioUrl());
  });

  async function speak(event: SubmitEvent) {
    event.preventDefault();
    const input = text().trim();
    if (busy() || !input || !voice()) return;
    setBusy(true);
    setError("");
    try {
      const response = await fetch("/v1/audio/speech", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ input, voice: voice() }),
        signal: controller.signal,
      });
      if (!response.ok) throw new Error((await response.text()) || "Could not generate speech.");
      const blob = await response.blob();
      if (controller.signal.aborted) return;
      if (audioUrl()) URL.revokeObjectURL(audioUrl());
      setAudioUrl(URL.createObjectURL(blob));
    } catch (error) {
      if (!controller.signal.aborted) setError(error instanceof Error ? error.message : "Could not generate speech.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <>
      <Header name="kale-tts" repository="kale-tts-rs" />
      <main>
        <form onSubmit={speak}>
          <label for="text">Text</label>
          <textarea id="text" value={text()} onInput={(event) => setText(event.currentTarget.value)} rows="6" required />
          <div class="actions">
            <label>Voice
              <select value={voice()} disabled={!voices().length || busy()} onChange={(event) => setVoice(event.currentTarget.value)}>
                <Show when={voices().length} fallback={<option value="">{loading() ? "Loading…" : "Unavailable"}</option>}>
                  <For each={voices()}>{(item) => <option value={item.id}>{item.name} · {item.accent}</option>}</For>
                </Show>
              </select>
            </label>
            <button type="submit" disabled={busy() || !text().trim() || !voices().length}>
              {busy() ? "Generating…" : "Generate speech"}
            </button>
          </div>
        </form>
        <Show when={error()}><p role="alert">{error()}</p></Show>
        <Show when={!loading() && !voices().length}><button class="secondary" onClick={loadVoices}>Retry loading voices</button></Show>
        <Show when={audioUrl()}>
          <div class="output">
            <audio autoplay controls src={audioUrl()} aria-label="Generated speech" />
            <a href={audioUrl()} download="kale-speech.wav">Download WAV</a>
          </div>
        </Show>
      </main>
    </>
  );
}
