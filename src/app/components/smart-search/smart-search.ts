import {Component, inject, signal} from '@angular/core';
import {open} from '@tauri-apps/plugin-dialog';
import {SmartSearchService} from "../../services/smart-search.service";
import {FormsModule} from "@angular/forms";
import {MatCard, MatCardContent} from "@angular/material/card";
import {MatIcon} from "@angular/material/icon";
import {MatButton, MatIconButton} from "@angular/material/button";
import {MatDivider, MatList, MatListItem} from "@angular/material/list";
import {MatFormField, MatInput, MatLabel} from "@angular/material/input";
import {MatChip, MatChipSet} from "@angular/material/chips";

interface SearchResult {
    id: number;
    file_path: string;
    snippet: string;
    score: number; // Релевантность (например, от 0.0 до 1.0)
}

@Component({
    selector: 'app-smart-search',
    imports: [
        FormsModule,
        MatCard,
        MatCardContent,
        MatIcon,
        MatButton,
        MatList,
        MatListItem,
        MatIconButton,
        MatDivider,
        MatFormField,
        MatLabel,
        MatInput,
        MatChipSet,
        MatChip
    ],
    templateUrl: './smart-search.html',
    styleUrl: './smart-search.css',
})
export class SmartSearch {
    // Список выбранных папок для сканирования
    scannedFolders = signal<string[]>([]);

    // Поиск
    searchQuery: string = '';
    searchResults = signal<SearchResult[]>([]);
    isSearching = signal<boolean>(false);
    isScanning = signal<boolean>(false);
    hasSearched = signal<boolean>(false); // Флаг: выполнялся ли поиск
    private smartSearchService = inject(SmartSearchService);
    private searchTimer: ReturnType<typeof setTimeout> | null = null;
    scanStatus = signal<'idle' | 'success' | 'error'>('idle');
    scanMessage = signal<string>('');

    // Выбор папок через Tauri Dialog с правильной типизацией
    async selectFolders() {
        try {
            const selected = await open({
                directory: true,
                multiple: true,
                title: 'Выберите папки для умного сканирования'
            }) as string | string[] | null;

            if (selected) {
                const paths: string[] = Array.isArray(selected) ? selected : [selected];

                // Обновляем сигнал через метод update, чтобы Angular сразу перерисовал экран
                this.scannedFolders.update(currentFolders => {
                    const updated = [...currentFolders];
                    for (const p of paths) {
                        if (p && !updated.includes(p)) {
                            updated.push(p);
                        }
                    }
                    return updated;
                });
            }
        } catch (error) {
            console.error('Ошибка выбора папок:', error);
        }
    }

    removeFolder(index: number) {
        // Безопасно удаляем элемент из сигнала
        this.scannedFolders.update(folders => folders.filter((_, i) => i !== index));
    }

    async startScanning() {
        const folders = this.scannedFolders();
        if (folders.length === 0) return;

        this.isScanning.set(true);
        this.scanStatus.set('idle'); // Сбрасываем статус перед новым запуском

        try {
            for (const folder of folders) {
                await this.smartSearchService.startNeuralScan(folder);
            }
            // Устанавливаем статус успеха
            this.scanStatus.set('success');
            this.scanMessage.set('Индексация успешно завершена!');
        } catch (error) {
            // Устанавливаем статус ошибки
            this.scanStatus.set('error');
            this.scanMessage.set('Произошла ошибка при сканировании');
            console.error(error);
        } finally {
            this.isScanning.set(false);

            // Прячем сообщение через 5 секунд, чтобы оно не висело вечно
            setTimeout(() => {
                this.scanStatus.set('idle');
            }, 5000);
        }
    }

    onSearchInput() {
        // Очищаем предыдущий таймер, если пользователь продолжает печатать
        if (this.searchTimer) {
            clearTimeout(this.searchTimer);
        }

        // Запускаем поиск через 400 мс после остановки ввода
        this.searchTimer = setTimeout(() => {
            this.onSearch().then();
        }, 400);
    }

    async onSearch() {
        this.hasSearched.set(true); // Отмечаем, что поиск был запущен

        if (!this.searchQuery.trim()) {
            this.searchResults.set([]);
            return;
        }

        this.isSearching.set(true);
        try {
            const results = await this.smartSearchService.search(this.searchQuery);
            this.searchResults.set(results);
        } catch (error) {
            console.error('Ошибка поиска:', error);
        } finally {
            this.isSearching.set(false);
        }
    }

    async revealFile(filePath: string) {
        try {
            await this.smartSearchService.revealInFolder(filePath);
        } catch (error) {
            console.error('Не удалось открыть папку с файлом:', error);
        }
    }
}
