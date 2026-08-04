const PREFIX = "mashiro.cache.";
const MEMORY_LIMIT = 8;
const MEMORY = new Map<string, unknown>();

function remember(key: string, data: unknown): void {
  MEMORY.delete(key);
  MEMORY.set(key, data);
  while (MEMORY.size > MEMORY_LIMIT) {
    const oldest = MEMORY.keys().next().value;
    if (oldest === undefined) break;
    MEMORY.delete(oldest);
  }
}

interface Entry<T> {
  at: number;
  data: T;
}

function storageKey(key: string): string {
  return `${PREFIX}${key}`;
}

export function readCache<T>(key: string): T | null {
  if (MEMORY.has(key)) {
    const cached = MEMORY.get(key) as T;
    remember(key, cached);
    return cached;
  }
  try {
    const raw = localStorage.getItem(storageKey(key));
    if (!raw) return null;
    const entry = JSON.parse(raw) as Entry<T>;
    remember(key, entry.data);
    return entry.data;
  } catch {
    return null;
  }
}

export function cachedAt(key: string): number | null {
  try {
    const raw = localStorage.getItem(storageKey(key));
    if (!raw) return null;
    return (JSON.parse(raw) as Entry<unknown>).at ?? null;
  } catch {
    return null;
  }
}

export function writeCache<T>(key: string, data: T): void {
  remember(key, data);
  try {
    const entry: Entry<T> = { at: Date.now(), data };
    localStorage.setItem(storageKey(key), JSON.stringify(entry));
  } catch {
    pruneCache();
  }
}

export function dropCache(key: string): void {
  MEMORY.delete(key);
  try {
    localStorage.removeItem(storageKey(key));
  } catch {}
}

export function clearCache(): void {
  MEMORY.clear();
  try {
    for (const key of Object.keys(localStorage)) {
      if (key.startsWith(PREFIX)) localStorage.removeItem(key);
    }
  } catch {}
}

export function releaseMemoryCache(): void {
  MEMORY.clear();
}

export function cacheSizeKb(): number {
  let total = 0;
  try {
    for (const key of Object.keys(localStorage)) {
      if (!key.startsWith(PREFIX)) continue;
      total += (localStorage.getItem(key) || "").length;
    }
  } catch {
    return 0;
  }
  return Math.round(total / 1024);
}

function pruneCache(): void {
  try {
    const entries = Object.keys(localStorage)
      .filter((key) => key.startsWith(PREFIX))
      .map((key) => {
        let at = 0;
        try {
          at = (JSON.parse(localStorage.getItem(key) || "{}") as Entry<unknown>)
            .at;
        } catch {
          at = 0;
        }
        return { key, at };
      })
      .sort((a, b) => a.at - b.at);
    for (const entry of entries.slice(0, Math.ceil(entries.length / 3))) {
      localStorage.removeItem(entry.key);
    }
  } catch {}
}

export interface SwrOptions<T> {
  onData: (data: T, source: "cache" | "network") => void;
  onError?: (error: unknown) => void;
  onSettled?: () => void;
  maxAge?: number;
}

export async function swr<T>(
  key: string,
  fetcher: () => Promise<T>,
  options: SwrOptions<T>,
): Promise<void> {
  const cached = readCache<T>(key);
  if (cached !== null) options.onData(cached, "cache");

  const age = cachedAt(key);
  if (
    cached !== null &&
    options.maxAge &&
    age &&
    Date.now() - age < options.maxAge
  ) {
    options.onSettled?.();
    return;
  }

  try {
    const fresh = await fetcher();
    const changed = JSON.stringify(fresh) !== JSON.stringify(cached);
    writeCache(key, fresh);
    if (changed) options.onData(fresh, "network");
  } catch (error) {
    options.onError?.(error);
  } finally {
    options.onSettled?.();
  }
}
