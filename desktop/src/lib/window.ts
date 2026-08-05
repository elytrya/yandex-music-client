type NativeWindow = {
  startDragging: () => Promise<void>;
  toggleMaximize: () => Promise<void>;
  setFocus?: () => Promise<void>;
  setFullscreen?: (value: boolean) => Promise<void>;
  isFullscreen?: () => Promise<boolean>;
};

async function currentWindow(): Promise<NativeWindow | null> {
  try {
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    return getCurrentWindow() as unknown as NativeWindow;
  } catch {
    return null;
  }
}

function interactive(target: EventTarget | null): boolean {
  const element = target as HTMLElement | null;
  return Boolean(
    element?.closest(
      "button, input, textarea, select, a, .icon-btn, .account, .win-controls, [data-no-drag]",
    ),
  );
}

export function releaseStuckPointer() {
  const up = () => new MouseEvent("mouseup", { bubbles: true, button: 0 });
  document.dispatchEvent(up());
  window.dispatchEvent(up());
}

export async function startWindowDrag(event: MouseEvent) {
  if (event.button !== 0 || interactive(event.target)) return;
  event.preventDefault();
  const win = await currentWindow();
  if (!win) return;
  try {
    await win.startDragging();
  } finally {
    releaseStuckPointer();
    try {
      await win.setFocus?.();
    } catch {}
  }
}

export async function toggleWindowMaximize(event: MouseEvent) {
  if (interactive(event.target)) return;
  event.preventDefault();
  await (await currentWindow())?.toggleMaximize();
}

export async function isNativeFullscreen(): Promise<boolean> {
  const win = await currentWindow();
  if (!win?.isFullscreen) return false;
  try {
    return await win.isFullscreen();
  } catch {
    return false;
  }
}

export async function setNativeFullscreen(value: boolean) {
  const win = await currentWindow();
  if (!win?.setFullscreen) return false;
  try {
    await win.setFullscreen(value);
    return true;
  } catch {
    return false;
  }
}
