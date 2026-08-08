const PREFIX = "mashiro.imgcache.";
const TTL = 1000 * 60 * 60 * 24 * 30;

interface Entry {
  data: string;
  at: number;
}

function read(key: string): Entry | null {
  try {
    const raw = localStorage.getItem(PREFIX + key);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Entry;
    if (!parsed?.data || typeof parsed.at !== "number") return null;
    if (Date.now() - parsed.at > TTL) {
      localStorage.removeItem(PREFIX + key);
      return null;
    }
    return parsed;
  } catch {
    return null;
  }
}

function write(key: string, data: string) {
  try {
    localStorage.setItem(
      PREFIX + key,
      JSON.stringify({ data, at: Date.now() }),
    );
  } catch {
    return;
  }
}

export function cachedImage(key: string): string | null {
  return read(key)?.data ?? null;
}

export async function loadCachedImage(
  key: string,
  url: string,
): Promise<string> {
  const hit = read(key);
  if (hit) return hit.data;
  try {
    const response = await fetch(url);
    if (!response.ok) return url;
    const blob = await response.blob();
    const data = await new Promise<string>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => resolve(String(reader.result));
      reader.onerror = () => reject(reader.error);
      reader.readAsDataURL(blob);
    });
    write(key, data);
    return data;
  } catch {
    return url;
  }
}
