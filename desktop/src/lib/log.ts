const DEV = Boolean(import.meta.env?.DEV);

function stamp(): string {
  const now = new Date();
  return now.toISOString().slice(11, 23);
}

function write(
  level: "log" | "warn" | "error",
  scope: string,
  args: unknown[],
): void {
  if (!DEV) return;
  const style =
    level === "error"
      ? "color:#ff6b6b"
      : level === "warn"
        ? "color:#ffb74d"
        : "color:#7ec4ff";
  console[level](`%c${stamp()} [${scope}]`, style, ...args);
}

export interface Logger {
  info: (...args: unknown[]) => void;
  warn: (...args: unknown[]) => void;
  error: (...args: unknown[]) => void;
  time: <T>(label: string, task: () => Promise<T>) => Promise<T>;
}

export function createLogger(scope: string): Logger {
  return {
    info: (...args) => write("log", scope, args),
    warn: (...args) => write("warn", scope, args),
    error: (...args) => write("error", scope, args),
    async time(label, task) {
      if (!DEV) return task();
      const started = performance.now();
      try {
        const result = await task();
        write("log", scope, [
          `${label}: ${Math.round(performance.now() - started)} ms`,
        ]);
        return result;
      } catch (error) {
        write("error", scope, [
          `${label} failed after ${Math.round(performance.now() - started)} ms`,
          error,
        ]);
        throw error;
      }
    },
  };
}

export const isDev = DEV;

export interface QualityReport {
  title?: string;
  codec: string | null;
  bitrate: number | null;
  source: string;
  requested: string;
}

export function logQuality(report: QualityReport): void {
  const codec = (report.codec || "unknown").toUpperCase();
  const rate =
    report.bitrate && report.bitrate > 0
      ? `${report.bitrate} kbps`
      : codec === "FLAC"
        ? "lossless"
        : "bitrate n/a";
  const lossless = codec === "FLAC" || codec === "ALAC" || codec === "WAV";
  const style = lossless
    ? "color:#4ade80;font-weight:600"
    : "color:#ffb74d;font-weight:600";
  console.info(
    `%c[quality]%c ${codec} ${rate} %c| запрошено: ${report.requested} | источник: ${report.source}${
      report.title ? ` | ${report.title}` : ""
    }`,
    style,
    "color:inherit;font-weight:600",
    "color:#a1a1a8",
  );
}
