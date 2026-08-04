import { convertFileSrc } from "@tauri-apps/api/core";

const SCHEME = "ymstream";

export function proxyStream(url: string): string {
  if (!url.startsWith("http://") && !url.startsWith("https://")) return url;
  try {
    return convertFileSrc(url, SCHEME);
  } catch {
    return url;
  }
}
