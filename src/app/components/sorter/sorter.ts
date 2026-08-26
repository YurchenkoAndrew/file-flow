import {Component, inject, signal} from '@angular/core';
import {MatButtonModule} from "@angular/material/button";
import {MatCardModule} from "@angular/material/card";
import {MatInputModule} from "@angular/material/input";
import {MatIconModule} from "@angular/material/icon";
import {MatProgressBarModule} from "@angular/material/progress-bar";
import {MatRadioModule} from "@angular/material/radio";
import {FormsModule} from "@angular/forms";
import {CommonModule} from "@angular/common";
import {MatFormFieldModule} from "@angular/material/form-field";
import {open} from '@tauri-apps/plugin-dialog';
import {SorterService} from "../../services/sorter.service";
import {SorterOptions} from "../../models/sorter.model";
import {StateService} from "../../services/state.service";

@Component({
    selector: 'app-sorter',
    imports: [
        CommonModule,
        FormsModule,
        MatCardModule,
        MatFormFieldModule,
        MatInputModule,
        MatButtonModule,
        MatRadioModule,
        MatProgressBarModule,
        MatIconModule
    ],
    templateUrl: './sorter.html',
    styleUrl: './sorter.css',
})
export class Sorter {

    title = 'Сортировщик';
    private sorterService = inject(SorterService);
    private sharedState = inject(StateService);

    // Пути и настройки
    sourcePath = signal<string>('');
    destinationPath = signal<string>('');
    operationMode: string = 'copy';

    // Состояние процесса
    isProcessing = signal<boolean>(false);
    progressValue = signal<number>(0);
    statusMessage = signal<string>('Готов к работе');

    // ДОБАВЛЕНО: Сигнал для хранения отчета
    sortingReport = signal<{success_count: number, error_count: number, errors?: string[]} | null>(null);

    ngOnInit() {
        const currentPath = this.sharedState.activePath();
        if (currentPath && !this.sourcePath()) {
            this.sourcePath.set(currentPath);
        }
    }

    async selectSource() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Выберите папку-источник'
            });
            if (selected && typeof selected === 'string') {
                this.sourcePath.set(selected);
            }
        } catch (error) {
            console.error('Ошибка выбора папки-источника:', error);
        }
    }

    async selectDestination() {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: 'Выберите папку назначения'
            });
            if (selected && typeof selected === 'string') {
                this.destinationPath.set(selected);
            }
        } catch (error) {
            console.error('Ошибка выбора папки назначения:', error);
        }
    }

    async startSorting() {
        if (!this.sourcePath() || !this.destinationPath()) {
            // Оставляем алерт только для валидации (или можно заменить на Snackbar)
            alert('Пожалуйста, выберите папки источника и назначения!');
            return;
        }

        // Очищаем предыдущий отчет перед новым запуском
        this.sortingReport.set(null);
        this.isProcessing.set(true);
        this.statusMessage.set('Выполняется сортировка и структурирование файлов...');

        try {
            const sessionId = this.sharedState.currentSessionId();

            const options: SorterOptions & { session_id?: number | null } = {
                source_path: this.sourcePath(),
                target_directory: this.destinationPath(),
                copy_files: this.operationMode === 'copy',
                group_by_year: true,
                session_id: sessionId
            };

            const result = await this.sorterService.startSorting(options);

            // Сохраняем результат в сигнал вместо вызова alert()
            this.sortingReport.set(result);
            this.statusMessage.set('Сортировка успешно завершена!');

            if (result.error_count > 0) {
                console.warn('Ошибки при сортировке:', result.errors);
            }
        } catch (error) {
            this.statusMessage.set(`Произошла ошибка при сортировке.`);
            // Если ошибка критическая (упал бэкенд), показываем её в отчете как 1 ошибку
            this.sortingReport.set({
                success_count: 0,
                error_count: 1,
                errors: [String(error)]
            });
        } finally {
            this.isProcessing.set(false);
        }
    }
}