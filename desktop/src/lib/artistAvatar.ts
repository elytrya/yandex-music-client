import { api } from "@/api/client";

const KEY = "mashiro.artistAvatars";
const memory = new Map<string, string | null>();
const pending = new Map<string, Promise<string | null>>();
let loaded = false;

function hydrate(): void {
  if (loaded) return;
  loaded = true;
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as Record<string, string | null>;
    for (const [id, url] of Object.entries(parsed)) memory.set(id, url);
  } catch {}
}

function persist(): void {
  try {
    const flat: Record<string, string | null> = {};
    for (const [id, url] of memory) flat[id] = url;
    localStorage.setItem(KEY, JSON.stringify(flat));
  } catch {
    memory.clear();
  }
}

export function cachedAvatar(id: string): string | null {
  hydrate();
  return memory.get(id) ?? null;
}

export async function fetchAvatar(id: string): Promise<string | null> {
  hydrate();
  if (!id) return null;
  if (memory.has(id)) return memory.get(id) ?? null;
  const running = pending.get(id);
  if (running) return running;

  const task = api
    .artist(id)
    .then((page) => page.cover_url ?? null)
    .catch(() => null)
    .then((url) => {
      memory.set(id, url);
      persist();
      pending.delete(id);
      return url;
    });

  pending.set(id, task);
  return task;
}

export async function fetchAvatars(
  ids: string[],
): Promise<Record<string, string | null>> {
  const unique = [...new Set(ids.filter(Boolean))].slice(0, 12);
  const out: Record<string, string | null> = {};
  await Promise.all(
    unique.map(async (id) => {
      out[id] = await fetchAvatar(id);
    }),
  );
  return out;
}
