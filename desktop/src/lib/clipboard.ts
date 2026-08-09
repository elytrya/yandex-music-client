import { Notify } from "quasar";

/**
 * Копирует текст и сама сообщает о результате.
 *
 * @param text что кладём в буфер; пустая строка — сразу отказ
 */
export async function copyText(
  text: string,
  done = "Скопировано",
  empty = "Нечего копировать",
): Promise<boolean> {
  if (!text) {
    Notify.create({ message: empty });
    return false;
  }

  try {
    await navigator.clipboard.writeText(text);
    Notify.create({ message: done });
    return true;
  } catch {
    Notify.create({ message: "Не удалось скопировать" });
    return false;
  }
}
