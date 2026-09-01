import {Component, inject, OnDestroy, OnInit, signal} from '@angular/core';
import {open} from '@tauri-apps/plugin-dialog';
import {SmartSearchService} from "../../services/smart-search.service";
import {FormsModule} from "@angular/forms";
import {MatCard, MatCardContent} from "@angular/material/card";
import {MatIcon} from "@angular/material/icon";
import {MatButton, MatIconButton} from "@angular/material/button";
import {MatDivider, MatList, MatListItem} from "@angular/material/list";
import {MatFormField, MatInput, MatLabel} from "@angular/material/input";
import {MatChip, MatChipSet} from "@angular/material/chips";
import {MAT_DIALOG_DATA, MatDialog, MatDialogModule} from "@angular/material/dialog";
import {MatProgressBar} from "@angular/material/progress-bar";
import {NeuroScanStatus} from "../../models/neuro-scanner.model";

interface SearchResult {
    id: number;
    file_path: string;
    snippet: string;
    score: number;
}

// Компонент диалогового окна для подтверждения удаления
@Component({
    selector: 'app-confirm-dialog',
    imports: [MatDialogModule, MatButton],
    template: `
        <h2 mat-dialog-title>Подтверждение удаления</h2>
        <mat-dialog-content>
            Вы действительно хотите удалить папку <br><strong>{{ data.folder }}</strong><br> из отслеживаемых? <br><br>
            Она больше не будет сканироваться, а все её проиндексированные документы будут удалены из базы поиска.
        </mat-dialog-content>
        <mat-dialog-actions align="end">
            <button mat-button mat-dialog-close>Отмена</button>
            <button mat-flat-button color="warn" [mat-dialog-close]="true">Удалить</button>
        </mat-dialog-actions>
    `
})
export class ConfirmDialogComponent {
    data = inject(MAT_DIALOG_DATA);
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
        MatChip,
        MatDialogModule,
        MatProgressBar,
        // Добавлен модуль диалоговых окон
    ],
    templateUrl: './smart-search.html',
    styleUrl: './smart-search.css',
})
export class SmartSearch implements OnInit, OnDestroy {
    scannedFolders = signal<string[]>([]);

    searchQuery: string = '';
    searchResults = signal<SearchResult[]>([]);
    isSearching = signal<boolean>(false);
    isScanning = signal<boolean>(false);
    hasSearched = signal<boolean>(false);

    private smartSearchService = inject(SmartSearchService);
    private dialog = inject(MatDialog); // Инжектируем сервис диалогов
    private searchTimer: ReturnType<typeof setTimeout> | null = null;
    private progressInterval: ReturnType<typeof setInterval> | null = null; // Таймер поллинга

    neuroScanStatus = signal<'idle' | 'success' | 'error'>('idle');
    // Это твой текущий сигнал, отвечающий за UI-состояние
    // ДОБАВЬ этот сигнал для хранения самих данных сканирования
    neuroScanData = signal<NeuroScanStatus | null>(null);
    scanMessage = signal<string>('');

    // Новые сигналы для прогресса
    scanProgress = signal<number>(0); // Сигнал для процентов

    async ngOnInit() {
        try {
            const folders = await this.smartSearchService.getWatchedFolders();
            this.scannedFolders.set(folders);
            await this.checkActiveScan(); // При заходе на страницу проверяем, не идет ли уже скан в фоне
        } catch (error) {
            console.error('Ошибка загрузки папок из базы:', error);
        }
    }

    ngOnDestroy() {
        if (this.progressInterval) {
            clearInterval(this.progressInterval);
        }
    }

    // Если скан уже идет, подхватываем его статус и блокируем интерфейс
    async checkActiveScan() {
        try {
            const status = await this.smartSearchService.getNeuralScanProgress();
            this.neuroScanData.set(status); // <-- Записываем данные в сигнал!

            if (status.is_running) {
                this.isScanning.set(true);
                this.updateProgressPercent(status.processed, status.total);
                this.startPolling();
            }
        } catch (e) {
            console.error('Ошибка проверки статуса', e);
        }
    }

    async startScanning() {
        const folders = this.scannedFolders();
        if (folders.length === 0) return;

        this.isScanning.set(true);
        this.neuroScanStatus.set('idle');
        this.scanProgress.set(0);

        try {
            // Передаем весь массив сразу. Не ждем завершения сканирования!
            await this.smartSearchService.startNeuralScan(folders);
            this.startPolling(); // Запускаем опрос прогресса
        } catch (error) {
            this.neuroScanStatus.set('error');
            this.scanMessage.set('Ошибка при запуске: Возможно, индексация уже идет.');
            this.isScanning.set(false);
            console.error(error);
        }
    }

    startPolling() {
        if (this.progressInterval) clearInterval(this.progressInterval);

        // Раз в секунду опрашиваем бэкенд о состоянии базы данных
        this.progressInterval = setInterval(async () => {
            try {
                const status = await this.smartSearchService.getNeuralScanProgress();
                this.neuroScanData.set(status); // <-- Записываем данные в сигнал!

                if (status.is_running) {
                    this.updateProgressPercent(status.processed, status.total);
                } else {
                    // Индексация завершилась
                    clearInterval(this.progressInterval!);
                    this.isScanning.set(false);
                    this.scanProgress.set(100);
                    this.neuroScanStatus.set('success');
                    this.scanMessage.set('Индексация успешно завершена!');
                    setTimeout(() => this.neuroScanStatus.set('idle'), 5000);
                }
            } catch (e) {
                console.error("Ошибка опроса прогресса", e);
            }
        }, 1000);
    }

    updateProgressPercent(processed: number, total: number) {
        if (total === 0) {
            this.scanProgress.set(0);
        } else {
            this.scanProgress.set(Math.round((processed / total) * 100));
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

    // Метод удаления с вызовом диалогового окна
    removeFolder(index: number) {
        const folderToRemove = this.scannedFolders()[index];
        if (!folderToRemove) return;

        // Открываем диалоговое окно
        const dialogRef = this.dialog.open(ConfirmDialogComponent, {
            data: {folder: folderToRemove},
            width: '450px'
        });

        // Ждем решения пользователя
        dialogRef.afterClosed().subscribe(async (confirmed) => {
            if (confirmed) {
                try {
                    // Удаляем из базы через бэкенд
                    await this.smartSearchService.removeWatchedFolder(folderToRemove);

                    // Удаляем из UI
                    this.scannedFolders.update(folders => folders.filter((_, i) => i !== index));
                } catch (error) {
                    console.error('Ошибка при удалении папки:', error);
                }
            }
        });
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