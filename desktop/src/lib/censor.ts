
const LIST_URL =
  "https://raw.githubusercontent.com/Hazzz895/FckCensorData/refs/heads/main/list.json";
const CACHE_KEY = "mashiro.censor.map";
const TTL = 1000 * 60 * 60 * 24; // сутки

let map: Record<string, string> = {};
let loaded = false;
let loading: Promise<void> | null = null;

function key(id: string): string {
  return id.split(":")[0];
}

function loadCache(): Record<string, string> | null {
  try {
    const raw = localStorage.getItem(CACHE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as {
      at: number;
      map: Record<string, string>;
    };
    if (!parsed || typeof parsed.map !== "object" || !parsed.map) return null;
    if (Date.now() - (parsed.at || 0) > TTL) return null;
    return parsed.map;
  } catch {
    return null;
  }
}

function persist(): void {
  try {
    localStorage.setItem(CACHE_KEY, JSON.stringify({ at: Date.now(), map }));
  } catch {
  }
}

export async function ensureCensorList(): Promise<void> {
  if (loaded) return;
  const cached = loadCache();
  if (cached) {
    map = cached;
    loaded = true;
    return;
  }
  if (loading) return loading;
  loading = fetch(LIST_URL)
    .then((r) =>
      r.ok ? r.json() : Promise.reject(new Error(String(r.status))),
    )
    .then((data: unknown) => {
      const tracks =
        data && typeof data === "object"
          ? ((data as { tracks?: Record<string, unknown> }).tracks ?? {})
          : {};
      const next: Record<string, string> = {};
      for (const [id, url] of Object.entries(tracks)) {
        if (typeof url === "string" && url) next[key(String(id))] = url;
      }
      map = next;
      loaded = true;
      persist();
    })
    .catch(() => {
      loaded = true;
    })
    .finally(() => {
      loading = null;
    });
  return loading;
}

export function isCensored(id: string | null | undefined): boolean {
  if (!id) return false;
  return Boolean(map[key(String(id))]);
}

export function censorUrl(id: string | null | undefined): string | null {
  if (!id) return null;
  return map[key(String(id))] ?? null;
}
