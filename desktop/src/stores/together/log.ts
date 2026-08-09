export const LOG_LIMIT = 300;

export interface TogetherLogEvent {
  time: string;
  scope: string;
  text: string;
}

export function formatEvent(event: TogetherLogEvent): string {
  return `${event.time} [${event.scope}] ${event.text}`;
}

export function formatLocal(scope: string, text: string): string {
  const now = new Date();
  const time = now.toTimeString().slice(0, 8);
  const ms = String(now.getMilliseconds()).padStart(3, "0");
  return `${time}.${ms} [${scope}] ${text}`;
}

export function append(lines: string[], line: string): string[] {
  const next = lines.length >= LOG_LIMIT ? lines.slice(1 - LOG_LIMIT) : lines;
  next.push(line);
  return next;
}
