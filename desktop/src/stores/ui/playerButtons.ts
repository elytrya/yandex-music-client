export type PlayerButtonId =
  | "shuffle"
  | "prev"
  | "play"
  | "next"
  | "repeat"
  | "like"
  | "dislike"
  | "sleep"
  | "mini"
  | "queue"
  | "lyrics"
  | "speed"
  | "quality"
  | "volume";

export type PlayerZone = "left" | "center" | "right" | "off";

export type PlayerButtonSlots = Record<PlayerButtonId, PlayerZone>;

export type MiniButtonId =
  | "shuffle"
  | "prev"
  | "play"
  | "next"
  | "repeat"
  | "like"
  | "dislike"
  | "lyrics"
  | "volume";

export type MiniButtonSlots = Record<MiniButtonId, boolean>;

export interface PlayerButtonMeta {
  id: PlayerButtonId;
  label: string;
  icon: string;
  hint: string;
  locked?: boolean;
}

export const playerButtonCatalog: PlayerButtonMeta[] = [
  {
    id: "play",
    label: "Play / Pause",
    icon: "play",
    hint: "Всегда в центре",
    locked: true,
  },
  { id: "prev", label: "Предыдущий", icon: "prev", hint: "Трек назад" },
  { id: "next", label: "Следующий", icon: "next", hint: "Трек вперёд" },
  {
    id: "shuffle",
    label: "Перемешать",
    icon: "shuffle",
    hint: "Случайный порядок",
  },
  {
    id: "repeat",
    label: "Повтор",
    icon: "repeat",
    hint: "Повтор трека или списка",
  },
  {
    id: "like",
    label: "Нравится",
    icon: "heart",
    hint: "Добавить в «Мне нравится»",
  },
  {
    id: "dislike",
    label: "Не нравится",
    icon: "heartOff",
    hint: "Больше не рекомендовать",
  },
  {
    id: "lyrics",
    label: "Текст песни",
    icon: "lyrics",
    hint: "Панель с текстом",
  },
  {
    id: "queue",
    label: "Очередь",
    icon: "queue",
    hint: "Список воспроизведения",
  },
  {
    id: "mini",
    label: "Мини-плеер",
    icon: "mini",
    hint: "Компактное окно поверх",
  },
  {
    id: "sleep",
    label: "Таймер сна",
    icon: "clock",
    hint: "Остановить через N минут",
  },
  {
    id: "speed",
    label: "Скорость",
    icon: "speed",
    hint: "Темп воспроизведения",
  },
  { id: "quality", label: "Качество", icon: "quality", hint: "Битрейт потока" },
  {
    id: "volume",
    label: "Громкость",
    icon: "volume",
    hint: "Кнопка и ползунок",
  },
];

export const miniButtonCatalog: Array<{
  id: MiniButtonId;
  label: string;
  icon: string;
  locked?: boolean;
}> = [
  { id: "play", label: "Play / Pause", icon: "play", locked: true },
  { id: "prev", label: "Предыдущий", icon: "prev", locked: true },
  { id: "next", label: "Следующий", icon: "next", locked: true },
  { id: "shuffle", label: "Перемешать", icon: "shuffle" },
  { id: "repeat", label: "Повтор", icon: "repeat" },
  { id: "like", label: "Нравится", icon: "heart" },
  { id: "dislike", label: "Не нравится", icon: "heartOff" },
  { id: "lyrics", label: "Текст песни", icon: "lyrics" },
  { id: "volume", label: "Громкость", icon: "volume" },
];

export const playerButtonOrder: PlayerButtonId[] = [
  "like",
  "dislike",
  "shuffle",
  "prev",
  "play",
  "next",
  "repeat",
  "lyrics",
  "queue",
  "mini",
  "sleep",
  "speed",
  "quality",
  "volume",
];

export const miniButtonOrder: MiniButtonId[] = [
  "like",
  "shuffle",
  "prev",
  "play",
  "next",
  "repeat",
  "dislike",
  "lyrics",
  "volume",
];

export const defaultPlayerButtonSlots: PlayerButtonSlots = {
  like: "left",
  dislike: "left",
  shuffle: "center",
  prev: "center",
  play: "center",
  next: "center",
  repeat: "center",
  lyrics: "right",
  queue: "right",
  mini: "right",
  sleep: "off",
  speed: "off",
  quality: "right",
  volume: "right",
};

export const defaultMiniButtons: MiniButtonSlots = {
  shuffle: false,
  prev: true,
  play: true,
  next: true,
  repeat: false,
  like: true,
  dislike: false,
  lyrics: false,
  volume: false,
};

export const playerZoneLabels: Array<{ value: PlayerZone; label: string }> = [
  { value: "left", label: "Слева" },
  { value: "center", label: "Центр" },
  { value: "right", label: "Справа" },
  { value: "off", label: "Скрыть" },
];
