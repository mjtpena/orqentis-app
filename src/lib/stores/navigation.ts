import { writable } from "svelte/store";
import type { Page } from "../types";

export const currentPage = writable<Page>("home");

function getInitialTheme(): "dark" | "light" {
  const saved = localStorage.getItem("orqentis-theme");
  if (saved === "light" || saved === "dark") return saved;
  return window.matchMedia("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

const initial = getInitialTheme();
document.documentElement.setAttribute("data-theme", initial);
export const theme = writable<"dark" | "light">(initial);

export function navigateTo(page: Page) {
  currentPage.set(page);
}

export function toggleTheme() {
  theme.update((t) => {
    const next = t === "dark" ? "light" : "dark";
    document.documentElement.setAttribute("data-theme", next);
    localStorage.setItem("orqentis-theme", next);
    return next;
  });
}
