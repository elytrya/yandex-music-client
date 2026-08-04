import { reactive } from "vue";
import { api } from "@/api/client";

const artistAi = reactive<Record<string, boolean>>({});
const requestedArtists = new Set<string>();
let artistQueue = new Set<string>();
let artistTimer: ReturnType<typeof setTimeout> | null = null;

async function flushArtists() {
  artistTimer = null;
  const ids = [...artistQueue];
  artistQueue = new Set();
  if (!ids.length) return;
  try {
    const results = await api.aiCheckArtists(ids);
    for (const r of results) artistAi[r.id] = r.ai;
  } catch {
  }
}

export function ensureAiArtists(ids: Array<string | null | undefined>): void {
  let scheduled = false;
  for (const id of ids) {
    if (!id || requestedArtists.has(id)) continue;
    requestedArtists.add(id);
    artistQueue.add(id);
    scheduled = true;
  }
  if (scheduled && artistTimer === null) {
    artistTimer = setTimeout(() => void flushArtists(), 120);
  }
}

export function isAiArtist(id: string | null | undefined): boolean {
  return !!id && artistAi[id] === true;
}

const trackAi = reactive<Record<string, boolean>>({});
const requestedTracks = new Set<string>();
let trackQueue = new Set<string>();
let trackTimer: ReturnType<typeof setTimeout> | null = null;

async function flushTracks() {
  trackTimer = null;
  const ids = [...trackQueue];
  trackQueue = new Set();
  if (!ids.length) return;
  try {
    const results = await api.aiCheckTracks(ids);
    for (const r of results) trackAi[r.id] = r.ai;
  } catch {
  }
}

export function ensureAiTracks(ids: Array<string | null | undefined>): void {
  let scheduled = false;
  for (const id of ids) {
    if (!id || requestedTracks.has(id)) continue;
    requestedTracks.add(id);
    trackQueue.add(id);
    scheduled = true;
  }
  if (scheduled && trackTimer === null) {
    trackTimer = setTimeout(() => void flushTracks(), 120);
  }
}

export function isAiTrack(id: string | null | undefined): boolean {
  return !!id && trackAi[id] === true;
}
