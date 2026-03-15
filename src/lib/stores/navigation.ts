import { writable } from 'svelte/store';
import type { Page } from '../types';

export const currentPage = writable<Page>('home');
export const theme = writable<'dark' | 'light'>('dark');

export function navigateTo(page: Page) {
  currentPage.set(page);
}

export function toggleTheme() {
  theme.update(t => {
    const next = t === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', next);
    return next;
  });
}
