import {Component, computed, inject, signal} from '@angular/core';
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
import {ScannerService} from "../../services/scanner.service";
import {SharedService} from "../../services/shared.service";
import {Color, LegendPosition, NgxChartsModule} from "@swimlane/ngx-charts";
import {ThemeService} from "../../services/theme.service";
import {StateService} from "../../services/state.service";
import {CleanupResponse, DuplicatesService} from "../../services/duplicates.service";

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
        MatTabLabel,
        NgxChartsModule
    ],
    templateUrl: './scanner.html',
    styleUrl: './scanner.css',
})
export class Scanner {
    // Сигналы Angular для реактивного состояния
    // 1. Инжектируем сервис тем
    private themeService = inject(ThemeService);
    private scannerService = inject(ScannerService);
    private sharedService = inject(SharedService);
    private readonly sharedState = inject(StateService);
    private duplicatesService = inject(DuplicatesService);
    isDeletingDuplicates = signal<boolean>(false)
    isLoading = signal<boolean>(false);
    selectedPath = signal<string>('');
    scanResult = signal<ScanResultSummary | null>(null);
    errorMessage = signal<string | null>(null);
    // ДОБАВЛЕНО: запоминаем активный таб и храним отчет
    activeTabIndex = signal<number>(0);
    cleanupReport = signal<CleanupResponse | null>(null);
    legendPosition: LegendPosition = LegendPosition.Right;
    // 2. Берем сигнал напрямую из сервиса, а не создаем локальный!
    isDarkMode = this.themeService.isDarkMode;
    // Теперь computed будет реактивно пересчитываться при изменении темы
    chartScheme = computed<string | Color>(() => {
        return this.isDarkMode() ? 'vivid' : 'cool';
    });

    // Словарь для красивых названий категорий на русском
    categoryNames: Record<FileCategory, string> = {
        Images: 'Изображения',
        Videos: 'Видео',
        Documents: 'Документы',
        Audios: 'Аудио',
        Archives: 'Архивы',
        Code: 'Код и скрипты',
        Software: 'Программы для ПК',
        Mobile: 'Мобильные (APK/IPA)',
        DiskImages: 'Образы дисков',
        DesignProjects: 'Дизайн (PSD/AI)',
        VideoProjects: 'Видеопроекты (Pr/Ae)',
        Fonts: 'Шрифты',

        Other: 'Прочее'
    };

    ngOnInit() {
        // Восстанавливаем путь в инпут, если он был сохранен
        const savedPath = this.sharedState.activePath();
        if (savedPath && !this.selectedPath()) {
            this.selectedPath.set(savedPath);
        }

        // Мгновенно возвращаем прошлые результаты сканирования на экран без повторного запуска
        const savedResult = this.sharedState.currentScanResult();
        if (savedResult && !this.scanResult()) {
            this.scanResult.set(savedResult);
        }
    }

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

    // И сам метод старта сканирования:
    // ОБНОВЛЕНО: добавлен флаг isRescan, чтобы не стирать отчет после удаления
    async startScan(path: string, isRescan: boolean = false) {
        this.isLoading.set(true);
        this.errorMessage.set(null);

        // Стираем старый отчет, только если это новое ручное сканирование
        if (!isRescan) {
            this.cleanupReport.set(null);
        }

        try {
            const result = await this.scannerService.scanDirectory(path);
            this.scanResult.set(result);
            if (result.session_id) {
                this.sharedState.setActiveSession(result.session_id, path, result);
            }
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

    // Нативная функция форматирования для выносок (стрелочек) графика
    pieLabelFormatting = (label: string): string => {
        // Получаем наши актуальные данные
        const chartData = this.getChartData();
        // Ищем текущую категорию по имени, чтобы достать её размер
        const item = chartData.find(d => d.name === label);

        if (item) {
            // Достаем настоящий размер (используем extra.realValue, если делали фикс для мелких файлов)
            const realSize = item.extra?.realValue ?? item.value;
            // Возвращаем красивую строку: "Категория (Размер)"
            return `${label} (${this.formatBytes(realSize)})`;
        }

        return label; // Фоллбэк, если ничего не нашлось
    }

    // Форматирование значения для тултипа графиков
    tooltipFormatting = (data: any): string => {
        const name = data.data?.name || data.data?.label || '';

        // Пытаемся взять честный размер из extra.realValue. Если его вдруг нет, берем стандартный value.
        const value = data.data?.extra?.realValue ?? data.value ?? data.data?.value ?? 0;

        return `${name}: ${this.formatBytes(value)}`;
    }

    // Преобразуем данные категорий для ngx-charts
    getChartData() {
        const res = this.scanResult();
        if (!res || !res.category_stats || res.total_size === 0) return [];

        const minVisualSize = res.total_size * 0.005;

        return res.category_stats
            .filter(stat => stat.files_count > 0 && stat.total_size > 0)
            .map(stat => {
                const baseName = this.categoryNames[stat.category] || stat.category;
                const formattedSize = this.formatBytes(stat.total_size);

                return {
                    // Теперь имя в легенде сразу будет содержать размер: "Видео (21.31 ГБ)"
                    name: `${baseName} (${formattedSize})`,
                    value: stat.total_size < minVisualSize ? minVisualSize : stat.total_size,
                    extra: {realValue: stat.total_size}
                };
            });
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

    // Открывает просмотр файла в проводнике по пути его расположения
    async showFileByPath(path: string): Promise<void> {
        this.errorMessage.set(null);
        try {
            await this.sharedService.showFileByPath(path);
            console.log(path);
        } catch (error) {
            this.errorMessage.set('Ошибка при открытии папки с файлом: ' + error);
        }
    }

    // Метод удаления дубликатов
    // ОБНОВЛЕНО: убран alert, результат пишется в сигнал, вызывается startScan с флагом
    async removeDuplicates() {
        const res = this.scanResult();
        if (!res || !res.duplicate_groups || res.duplicate_groups.length === 0) return;

        const currentSessionId = this.sharedState.currentSessionId();
        if (!currentSessionId) {
            this.errorMessage.set('Нет активной сессии. Просканируйте папку заново.');
            return;
        }

        const confirmed = confirm('Вы уверены, что хотите удалить дубликаты? Самые старые версии файлов будут сохранены, а их копии удалены с диска.');
        if (!confirmed) return;

        this.isDeletingDuplicates.set(true);
        this.errorMessage.set(null);
        this.cleanupReport.set(null); // Прячем прошлый отчет, если удаляем снова

        try {
            const result = await this.duplicatesService.removeDuplicates(currentSessionId, res.duplicate_groups);

            // Сохраняем красивый отчет
            this.cleanupReport.set(result);

            // Перезапускаем сканирование тихо (isRescan = true)
            if (this.selectedPath()) {
                await this.startScan(this.selectedPath(), true);
            }
        } catch (error) {
            this.errorMessage.set(`Ошибка при удалении дубликатов: ${error}`);
        } finally {
            this.isDeletingDuplicates.set(false);
        }
    }
}
