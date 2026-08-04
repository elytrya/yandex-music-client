export type HotkeyAction =
  | "toggle"
  | "mute"
  | "seekForward"
  | "seekBackward"
  | "volumeUp"
  | "volumeDown"
  | "like"
  | "dislike"
  | "repeat"
  | "shuffle"
  | "next"
  | "prev"
  | "lyrics"
  | "queue";

export interface HotkeyMeta {
  action: HotkeyAction;
  label: string;
  keys: string[];
  global: string;
}

export const HOTKEYS: HotkeyMeta[] = [
  {
    action: "toggle",
    label: "Включить музыку / поставить на паузу",
    keys: ["Space", "K"],
    global: "MediaPlayPause",
  },
  {
    action: "mute",
    label: "Отключить / включить звук",
    keys: ["M"],
    global: "",
  },
  {
    action: "seekForward",
    label: "Промотать вперёд",
    keys: ["ArrowRight", "L"],
    global: "",
  },
  {
    action: "seekBackward",
    label: "Промотать назад",
    keys: ["ArrowLeft", "J"],
    global: "",
  },
  {
    action: "volumeUp",
    label: "Увеличить громкость",
    keys: ["ArrowUp"],
    global: "",
  },
  {
    action: "volumeDown",
    label: "Уменьшить громкость",
    keys: ["ArrowDown"],
    global: "",
  },
  {
    action: "like",
    label: "Лайк",
    keys: ["F"],
    global: "CommandOrControl+Alt+L",
  },
  { action: "dislike", label: "Дизлайк", keys: ["D"], global: "" },
  {
    action: "repeat",
    label: "Переключение режима повтора",
    keys: ["R"],
    global: "",
  },
  {
    action: "shuffle",
    label: "Переключение режима «в случайном порядке»",
    keys: ["S"],
    global: "",
  },
  {
    action: "next",
    label: "Переключить на следующий трек",
    keys: ["N"],
    global: "MediaTrackNext",
  },
  {
    action: "prev",
    label: "Переключить на предыдущий трек",
    keys: ["P"],
    global: "MediaTrackPrevious",
  },
  {
    action: "lyrics",
    label: "Открыть / закрыть текст песни",
    keys: ["W"],
    global: "",
  },
  {
    action: "queue",
    label: "Показать / скрыть очередь",
    keys: ["Q"],
    global: "",
  },
];

export type HotkeyMap = Record<HotkeyAction, string[]>;
export type GlobalHotkeyMap = Record<HotkeyAction, string>;

export function defaultLocalMap(): HotkeyMap {
  const map = {} as HotkeyMap;
  for (const meta of HOTKEYS) map[meta.action] = [...meta.keys];
  return map;
}

export function defaultGlobalMap(): GlobalHotkeyMap {
  const map = {} as GlobalHotkeyMap;
  for (const meta of HOTKEYS) map[meta.action] = meta.global;
  return map;
}
