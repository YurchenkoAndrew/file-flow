import {Component, inject, signal} from '@angular/core';
import {FileCategory, ScanResultSummary} from "../../models/scanner.model";
import {open} from '@tauri-apps/plugin-dialog';
import {MatCardModule} from "@angular/material/card";
import {MatIconModule} from "@angular/material/icon";
import {MatInputModule} from "@angular/material/input";
import {MatButtonModule} from "@angular/material/button";
import {CommonModule} from "@angular/common";
import {FormsModule} from "@angular/forms";
import {MatFormFieldModule} from "@angular/material/form-field";
import {MatRadioModule} from "@angular/material/radio";
import {MatProgressBarModule} from "@angular/material/progress-bar";
import {MatTab, MatTabGroup, MatTabLabel} from "@angular/material/tabs";
import {ScannerService} from "../../services/scanner";

@Component({
    selector: 'app-scanner',
    imports: [
        CommonModule,
        FormsModule,
        MatCardModule,
        MatFormFieldModule,
        MatInputModule,
        MatButtonModule,
        MatRadioModule,
        MatProgressBarModule,
        MatIconModule,
        MatTabGroup,
        MatTab,
        MatTabLabel
    ],
    templateUrl: './scanner.html',
    styleUrl: './scanner.css',
})
export class Scanner {
    private scannerService = inject(ScannerService);
    // Сигналы Angular для реактивного состояния
    isLoading = signal<boolean>(false);
    selectedPath = signal<string>('');
    scanResult = signal<ScanResultSummary | null>(null);
    errorMessage = signal<string | null>(null);
    // Активная вкладка ('overview' | 'heavy' | 'duplicates')
    // activeTab = signal<string>('overview');

    // Словарь для красивых названий категорий на русском
    categoryNames: Record<FileCategory, string> = {
        Image: 'Изображения',
        Video: 'Видео',
        Document: 'Документы',
        Audio: 'Аудио',
        Archive: 'Архивы',
        Code: 'Код и скрипты',
        Software: 'Программы для ПК',
        Mobile: 'Мобильные (APK/IPA)',
        DiskImages: 'Образы дисков',
        Fonts: 'Шрифты',
        Other: 'Другое'
    };

    // Метод выбора папки через нативный диалог Tauri
    async selectFolder() {
        try {
            const folderPath = await open({
                directory: true,
                multiple: false,
                title: 'Выберите папку для сканирования'
            });

            if (folderPath && typeof folderPath === 'string') {
                this.selectedPath.set(folderPath); // Просто заполняем путь в инпут, сканирование по кнопке!
            }
        } catch (error) {
            this.errorMessage.set(`Ошибка выбора папки: ${error}`);
        }
    }

    // И сам метод старта сканирования выглядит чисто:
    async startScan(path: string) {
        this.isLoading.set(true);
        this.errorMessage.set(null);

        try {
            const result = await this.scannerService.scanDirectory(path);
            this.scanResult.set(result);
        } catch (error) {
            this.errorMessage.set(`Ошибка при сканировании: ${error}`);
        } finally {
            this.isLoading.set(false);
        }
    }

    // Утилита для красивого отображения байтов (ГБ, МБ, КБ)
    formatBytes(bytes: number): string {
        if (bytes === 0) return '0 Б';
        const k = 1024;
        const sizes = ['Б', 'КБ', 'МБ', 'ГБ', 'ТБ'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    // Считает общее количество файлов-дубликатов (все лишние копии)
    getTotalDuplicatesCount(): number {
        const res = this.scanResult();
        if (!res || !res.duplicate_groups) return 0;

        return res.duplicate_groups.reduce((acc, group) => {
            // В каждой группе первый файл — оригинал, остальные (group.files.length - 1) — дубликаты
            return acc + Math.max(0, group.files.length - 1);
        }, 0);
    }
}
