export const audio = new Audio();
audio.preload = "auto";
audio.crossOrigin = "anonymous";

let sourceToken = 0;

export function nextSourceToken(): number {
  sourceToken += 1;
  return sourceToken;
}

export function isCurrentToken(token: number): boolean {
  return token === sourceToken;
}

export async function safePlay(token?: number): Promise<void> {
  try {
    await audio.play();
  } catch (error) {
    const name = (error as DOMException | undefined)?.name;
    if (name === "AbortError") return;
    if (token !== undefined && !isCurrentToken(token)) return;
    throw error;
  }
}

let fadeTimer: number | null = null;

function stopFade(): void {
  if (fadeTimer !== null) {
    clearInterval(fadeTimer);
    fadeTimer = null;
  }
}

function ramp(from: number, to: number, ms: number): Promise<void> {
  stopFade();
  if (ms <= 0) {
    audio.volume = Math.max(0, Math.min(1, to));
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    const step = 25;
    const started = Date.now();
    audio.volume = Math.max(0, Math.min(1, from));
    fadeTimer = window.setInterval(() => {
      const t = Math.min(1, (Date.now() - started) / ms);
      audio.volume = Math.max(0, Math.min(1, from + (to - from) * t));
      if (t >= 1) {
        stopFade();
        resolve();
      }
    }, step);
  });
}

export async function fadeOut(ms: number): Promise<void> {
  if (ms <= 0 || audio.paused) return;
  await ramp(audio.volume, 0, ms);
}

export async function fadeIn(target: number, ms: number): Promise<void> {
  if (ms <= 0) {
    stopFade();
    audio.volume = Math.max(0, Math.min(1, target));
    return;
  }
  await ramp(0, target, ms);
}

export function cancelFade(target: number): void {
  stopFade();
  audio.volume = Math.max(0, Math.min(1, target));
}

export const EQ_BANDS = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000];

interface Graph {
  context: AudioContext;
  source: MediaElementAudioSourceNode;
  preamp: GainNode;
  filters: BiquadFilterNode[];
  routed: boolean;
}

let graph: Graph | null = null;
let failed = false;

function route(current: Graph, throughFilters: boolean): void {
  if (current.routed === throughFilters) return;
  current.routed = throughFilters;
  try {
    current.source.disconnect();
  } catch {}
  if (throughFilters) current.source.connect(current.preamp);
  else current.source.connect(current.context.destination);
  if (analyser) {
    try {
      current.source.connect(analyser);
    } catch {}
  }
}

function build(): Graph | null {
  if (graph) return graph;
  if (failed) return null;
  try {
    const context = new AudioContext({ latencyHint: "playback" });
    const source = context.createMediaElementSource(audio);
    const preamp = context.createGain();
    const filters = EQ_BANDS.map((frequency, index) => {
      const filter = context.createBiquadFilter();
      filter.type =
        index === 0
          ? "lowshelf"
          : index === EQ_BANDS.length - 1
            ? "highshelf"
            : "peaking";
      filter.frequency.value = frequency;
      filter.Q.value = 1.1;
      filter.gain.value = 0;
      return filter;
    });

    let node: AudioNode = preamp;
    for (const filter of filters) {
      node.connect(filter);
      node = filter;
    }
    node.connect(context.destination);

    graph = { context, source, preamp, filters, routed: false };
    source.connect(context.destination);
    void context.resume().catch(() => undefined);
    return graph;
  } catch {
    failed = true;
    return null;
  }
}

export function equalizerAvailable(): boolean {
  return !failed;
}

export function applyEqualizer(
  enabled: boolean,
  gains: number[],
  preampDb: number,
): boolean {
  if (!enabled && !graph) return true;
  const current = build();
  if (!current) return false;

  void current.context.resume().catch(() => undefined);
  route(current, enabled);
  const preampGain = enabled ? Math.pow(10, preampDb / 20) : 1;
  current.preamp.gain.value = preampGain;
  current.filters.forEach((filter, index) => {
    filter.gain.value = enabled ? (gains[index] ?? 0) : 0;
  });
  return true;
}

export function resumeAudioContext(): void {
  if (graph) void graph.context.resume().catch(() => undefined);
}

let analyser: AnalyserNode | null = null;
let levelBuffer: Float32Array | null = null;

export function ensureAnalyser(): boolean {
  const current = build();
  if (!current) return false;
  if (current.context.state === "suspended") {
    void current.context.resume().catch(() => undefined);
  }
  if (!analyser) {
    analyser = current.context.createAnalyser();
    analyser.fftSize = 1024;
    levelBuffer = new Float32Array(analyser.fftSize);
    current.source.connect(analyser);
  }
  return true;
}

let freqBuffer: Uint8Array | null = null;

export function frequencyData(): Uint8Array | null {
  if (!ensureAnalyser()) return null;
  if (!analyser) return null;
  if (!freqBuffer || freqBuffer.length !== analyser.frequencyBinCount) {
    freqBuffer = new Uint8Array(analyser.frequencyBinCount);
  }
  analyser.getByteFrequencyData(freqBuffer);
  return freqBuffer;
}

export function currentLevel(): number {
  if (!analyser || !levelBuffer) {
    if (!ensureAnalyser()) return 1;
  }
  if (!analyser || !levelBuffer) return 1;
  analyser.getFloatTimeDomainData(levelBuffer);
  let sum = 0;
  for (let i = 0; i < levelBuffer.length; i++) {
    const v = levelBuffer[i] ?? 0;
    sum += v * v;
  }
  return Math.sqrt(sum / levelBuffer.length);
}

let gestureArmed = false;

export function resumeOnGesture(run: () => void): void {
  if (gestureArmed) return;
  gestureArmed = true;
  const fire = () => {
    gestureArmed = false;
    window.removeEventListener("pointerdown", fire);
    window.removeEventListener("keydown", fire);
    run();
  };
  window.addEventListener("pointerdown", fire, { once: true });
  window.addEventListener("keydown", fire, { once: true });
}
