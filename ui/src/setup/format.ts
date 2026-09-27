// Sizes and times the way the rest of the app writes them: decimal GB, "5 min left".

export function gb(bytes: number): string {
  const g = bytes / 1e9;
  if (g >= 100) return `${g.toFixed(0)} GB`;
  if (g >= 0.1) return `${g.toFixed(1)} GB`;
  return `${Math.max(1, Math.round(bytes / 1e6))} MB`;
}

/** GiB of memory, as Windows shows it. */
export function memory(bytes: number): string {
  return `${Math.round(bytes / 1024 ** 3)} GB`;
}

export function speed(bytesPerSec: number): string {
  return `${(bytesPerSec / 1e6).toFixed(bytesPerSec >= 1e8 ? 0 : 1)} MB/s`;
}

export function timeLeft(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) return "";
  if (seconds < 60) return "less than a minute left";
  const min = Math.round(seconds / 60);
  if (min < 60) return `${min} min left`;
  const h = Math.floor(min / 60);
  return `${h} h ${min % 60} min left`;
}
