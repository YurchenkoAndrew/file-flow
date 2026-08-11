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

    // Пути и настройки (объявлены как сигналы для мгновенного обновления интерфейса)
    sourcePath = signal<string>('');
    destinationPath = signal<string>('');
    operationMode: string = 'copy'; // 'copy' или 'move'

    // Состояние процесса
    isProcessing = signal<boolean>(false);
    progressValue = signal<number>(0);
    statusMessage = signal<string>('Готов к работе');
    private sharedState = inject(StateService);

    ngOnInit() {
        // Если в состоянии уже есть активный путь от сканера, автоматически подставляем его в сортер
        const currentPath = this.sharedState.activePath();
        if (currentPath && !this.sourcePath()) {
            this.sourcePath.set(currentPath);
        }
    }


    // Выбор папки-источника через нативный диалог Tauri
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

    // Выбор папки-назначения через нативный диалог Tauri
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

    // Запуск сортировки
    async startSorting() {
        if (!this.sourcePath() || !this.destinationPath()) {
            alert('Пожалуйста, выберите папки источника и назначения!');
            return;
        }

        this.isProcessing.set(true);
        this.statusMessage.set('Выполняется сортировка и структурирование файлов...');

        try {
            // Достаем currentSessionId из глобального состояния, если оно есть
            const sessionId = this.sharedState.currentSessionId();

            const options: SorterOptions & { session_id?: number | null } = {
                source_path: this.sourcePath(),
                target_directory: this.destinationPath(),
                copy_files: this.operationMode === 'copy',
                group_by_year: true,
                session_id: sessionId // Передаем ID сессии на бэкенд!
            };

            const result = await this.sorterService.startSorting(options);

            // Красивый итог операции
            const actionText = this.operationMode === 'copy' ? 'скопировано' : 'перенесено';
            this.statusMessage.set(`Готово! Успешно ${actionText}: ${result.success_count}, ошибок: ${result.error_count}`);

            // Показываем детальный отчет пользователю
            alert(
                `📊 Отчет о сортировке:\n\n` +
                `• Успешно обработано: ${result.success_count}\n` +
                `• Ошибок: ${result.error_count}` +
                (result.error_count > 0 ? `\n\nПроверьте консоль для деталей по ошибкам.` : '')
            );

            if (result.error_count > 0) {
                console.warn('Ошибки при сортировке:', result.errors);
            }
        } catch (error) {
            this.statusMessage.set(`Ошибка при сортировке: ${error}`);
            alert(`Не удалось завершить сортировку:\n${error}`);
        } finally {
            this.isProcessing.set(false);
        }
    }
}