import {inject, PLATFORM_ID, Service, signal} from '@angular/core';
import {isPlatformBrowser} from "@angular/common";

@Service()
export class Theme {
    private platformId = inject(PLATFORM_ID);
    // Сигнал для отслеживания состояния темной темы
    public isDarkMode = signal<boolean>(false);
    // Сигнал: включен ли автоматический режим (по умолчанию true, если нет ручных настроек)
    public isAutoTheme = signal<boolean>(true);
    private mediaQuery: MediaQueryList | null = null;

    constructor() {
        if (isPlatformBrowser(this.platformId)) {
            const savedTheme = localStorage.getItem('user_theme_choice');

            if (savedTheme) {
                // Если пользователь ранее фиксировал тему вручную
                this.isAutoTheme.set(false);
                this.setTheme(savedTheme === 'dark');
            } else {
                // Режим Авто
                this.isAutoTheme.set(true);
                this.initAutoTheme();
            }
        }
    }

    // Переключение режима «Авто <-> Ручной» (например, через switch-тумблер)
    public setAutoMode(isAuto: boolean) {
        this.isAutoTheme.set(isAuto);

        if (isAuto) {
            // Стираем ручной выбор из памяти
            if (isPlatformBrowser(this.platformId)) {
                localStorage.removeItem('user_theme_choice');
            }
            // Возвращаем системную тему
            this.initAutoTheme();
        } else {
            // При переходе из авто в ручной фиксируем текущую тему как ручную
            this.saveCurrentAsManual();
        }
    }

    // Клик по кнопке смены темы (работает только в ручном режиме)
    public toggleManualTheme() {
        if (this.isAutoTheme()) return; // В авто-режиме кнопка не должна переключать вручную

        const nextMode = !this.isDarkMode();
        this.setTheme(nextMode);

        if (isPlatformBrowser(this.platformId)) {
            localStorage.setItem('user_theme_choice', nextMode ? 'dark' : 'light');
        }
    }

    private initAutoTheme() {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        this.setTheme(prefersDark);

        if (!this.mediaQuery && isPlatformBrowser(this.platformId)) {
            this.mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            this.mediaQuery.addEventListener('change', (e) => {
                // Слушаем систему ТОЛЬКО если активен авто-режим
                if (this.isAutoTheme()) {
                    this.setTheme(e.matches);
                }
            });
        }
    }

    private saveCurrentAsManual() {
        const currentDark = this.isDarkMode();
        if (isPlatformBrowser(this.platformId)) {
            localStorage.setItem('user_theme_choice', currentDark ? 'dark' : 'light');
        }
    }

    private setTheme(isDark: boolean) {
        this.isDarkMode.set(isDark);
        if (isPlatformBrowser(this.platformId)) {
            const body = document.body;
            if (isDark) {
                body.classList.add('dark-theme');
            } else {
                body.classList.remove('dark-theme');
            }
        }
    }
}
