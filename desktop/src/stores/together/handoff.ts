import { HANDOFF_GAP, HANDOFF_RETRIES } from "./protocol";

export function wait(ms: number): Promise<void> {
  return new Promise((resolve) => window.setTimeout(resolve, ms));
}

export interface ReconnectIo {
  join: (address: string) => Promise<void>;
  joined: () => boolean;
  note: (text: string) => void;
}

/**
 * Новый хост поднимает комнату не мгновенно, поэтому стучимся несколько раз.
 *
 * @param target адрес вида ip:port
 */
export async function reconnect(
  target: string,
  io: ReconnectIo,
): Promise<boolean> {
  for (let attempt = 1; attempt <= HANDOFF_RETRIES; attempt += 1) {
    await io.join(target);
    if (io.joined()) return true;

    io.note(`попытка ${attempt}: ${target} ещё не отвечает`);
    await wait(HANDOFF_GAP);
  }

  return false;
}
