export interface Place {
  x: number;
  y: number;
}

const EDGE = 8;

export function loadPlace(key: string): Place | null {
  try {
    const raw = localStorage.getItem(key);
    if (!raw) return null;

    const parsed = JSON.parse(raw) as Partial<Place>;
    if (typeof parsed.x !== "number" || typeof parsed.y !== "number") {
      return null;
    }
    return { x: parsed.x, y: parsed.y };
  } catch {
    return null;
  }
}

export function savePlace(key: string, place: Place | null): void {
  try {
    if (place) localStorage.setItem(key, JSON.stringify(place));
    else localStorage.removeItem(key);
  } catch {}
}

/** Не даём утащить окошко за край окна. */
export function clampPlace(place: Place, el: HTMLElement | null): Place {
  const box = el?.getBoundingClientRect();
  const width = box?.width ?? 280;
  const height = box?.height ?? 44;

  const maxX = Math.max(EDGE, window.innerWidth - width - EDGE);
  const maxY = Math.max(EDGE, window.innerHeight - height - EDGE);

  return {
    x: Math.min(Math.max(EDGE, place.x), maxX),
    y: Math.min(Math.max(EDGE, place.y), maxY),
  };
}
