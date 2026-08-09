import type { TogetherPeer } from "@/api/together";

export function keepAlive(ids: number[], peers: TogetherPeer[]): number[] {
  const alive = new Set(peers.map((peer) => peer.id));
  return ids.filter((id) => alive.has(id));
}

export function toggleId(ids: number[], id: number): number[] {
  return ids.includes(id) ? ids.filter((item) => item !== id) : [...ids, id];
}

export function withId(ids: number[], id: number, on: boolean): number[] {
  if (on) return ids.includes(id) ? ids : [...ids, id];
  return ids.filter((item) => item !== id);
}

export function nicksOf(ids: number[], peers: TogetherPeer[]): string[] {
  return peers.filter((peer) => ids.includes(peer.id)).map((peer) => peer.nick);
}
