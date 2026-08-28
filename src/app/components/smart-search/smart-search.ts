import {Component, inject, signal, OnInit} from '@angular/core'; // ДОБАВЛЕН OnInit
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
    score: number;
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
export class SmartSearch implements OnInit { // ДОБАВЛЕН implements OnInit
    scannedFolders = signal<string[]>([]);

    searchQuery: string = '';
    searchResults = signal<SearchResult[]>([]);
    isSearching = signal<boolean>(false);
    isScanning = signal<boolean>(false);
    hasSearched = signal<boolean>(false);
    private smartSearchService = inject(SmartSearchService);
    private searchTimer: ReturnType<typeof setTimeout> | null = null;
    scanStatus = signal<'idle' | 'success' | 'error'>('idle');
    scanMessage = signal<string>('');

    // ДОБАВЛЕНО: Загрузка папок при открытии компонента
    async ngOnInit() {
        try {
            const folders = await this.smartSearchService.getWatchedFolders();
            this.scannedFolders.set(folders);
        } catch (error) {
            console.error('Ошибка загрузки папок из базы:', error);
        }
    }

    async selectFolders() {
        try {
            const selected = await open({
                directory: true,
                multiple: true,
                title: 'Выберите папки для умного сканирования'
            }) as string | string[] | null;

            if (selected) {
                const paths: string[] = Array.isArray(selected) ? selected : [selected];

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

    // ИЗМЕНЕНО: Теперь папка удаляется и из базы данных
    async removeFolder(index: number) {
        const folderToRemove = this.scannedFolders()[index];
        if (!folderToRemove) return;

        try {
            // Удаляем из базы через бэкенд
            await this.smartSearchService.removeWatchedFolder(folderToRemove);

            // Удаляем из UI
            this.scannedFolders.update(folders => folders.filter((_, i) => i !== index));
        } catch (error) {
            console.error('Ошибка при удалении папки:', error);
        }
    }

    async startScanning() {
        const folders = this.scannedFolders();
        if (folders.length === 0) return;

        this.isScanning.set(true);
        this.scanStatus.set('idle');

        try {
            for (const folder of folders) {
                await this.smartSearchService.startNeuralScan(folder);
            }
            this.scanStatus.set('success');
            this.scanMessage.set('Индексация успешно завершена!');
        } catch (error) {
            this.scanStatus.set('error');
            this.scanMessage.set('Произошла ошибка при сканировании');
            console.error(error);
        } finally {
            this.isScanning.set(false);
            setTimeout(() => {
                this.scanStatus.set('idle');
            }, 5000);
        }
    }

    onSearchInput() {
        if (this.searchTimer) {
            clearTimeout(this.searchTimer);
        }
        this.searchTimer = setTimeout(() => {
            this.onSearch().then();
        }, 400);
    }

    async onSearch() {
        this.hasSearched.set(true);

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